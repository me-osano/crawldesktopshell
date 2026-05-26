use std::path::Path;
use std::sync::Arc;

use mail_parser::MessageParser;
use sqlx::Row;
use tokio::sync::broadcast;

use crawl_ipc::commands::CrawlCommand;
use crawl_ipc::events::MailEvent;
use crawl_ipc::protocol::error_code;
use crawl_ipc::types::{MailErrorCode, MailResponse, MessageFull, SaveAttachment};
use crawl_ipc::{CrawlEvent, CrawlResponse, IpcServer, RequestDispatcher};

use crate::imap::ImapSession;
use crate::{accounts::AccountManager, store::Store};

pub async fn serve(
    path: impl AsRef<Path>,
    store: Arc<Store>,
    accounts: Arc<AccountManager>,
    push_tx: broadcast::Sender<CrawlEvent>,
) -> anyhow::Result<()> {
    let socket_path = path.as_ref().to_path_buf();
    let mut server = IpcServer::new(socket_path, push_tx.clone());

    let dispatcher: RequestDispatcher = Arc::new(move |method, params, id| {
        let store = Arc::clone(&store);
        let accounts = Arc::clone(&accounts);
        let event_tx = push_tx.clone();
        Box::pin(async move {
            let mut command_value = serde_json::json!({"method": method.clone()});
            let include_params = !params.is_null()
                && !params
                    .as_object()
                    .map(|obj| obj.is_empty())
                    .unwrap_or(false);
            if include_params {
                if let serde_json::Value::Object(ref mut obj) = command_value {
                    obj.insert("params".to_string(), params.clone());
                }
            }

            let command: CrawlCommand = match serde_json::from_value(command_value) {
                Ok(cmd) => cmd,
                Err(_) => {
                    return CrawlResponse::error(
                        id,
                        error_code::INVALID_PARAMS,
                        "Invalid mail command",
                    );
                }
            };

            match command {
                CrawlCommand::ListAccounts => {
                    respond(
                        id,
                        store
                            .accounts()
                            .list_accounts()
                            .await
                            .map(|accounts| MailResponse::AccountList { accounts }),
                    )
                    .await
                }
                CrawlCommand::AddAccount(payload) => {
                    match store.accounts().add_account(&payload).await {
                        Ok(account_id) => {
                            let _ = event_tx.send(CrawlEvent::Mail(MailEvent::AccountAdded {
                                account_id: account_id.clone(),
                                display_name: payload.display_name,
                                email: payload.email,
                            }));
                            ok(id, serde_json::json!({"account_id": account_id}))
                        }
                        Err(err) => err_response(id, MailErrorCode::DbError, &err.to_string()),
                    }
                }
                CrawlCommand::RemoveAccount { account_id } => {
                    respond(
                        id,
                        store
                            .accounts()
                            .delete_account(&account_id)
                            .await
                            .map(|_| MailResponse::Ok),
                    )
                    .await
                }
                CrawlCommand::ListFolders { account_id } => {
                    respond(
                        id,
                        store
                            .folders()
                            .list_folders(&account_id)
                            .await
                            .map(|folders| MailResponse::FolderList { folders }),
                    )
                    .await
                }
                CrawlCommand::SelectFolder { .. } => ok(
                    id,
                    serde_json::to_value(MailResponse::Ok).unwrap_or_default(),
                ),
                CrawlCommand::ListMessages(params) => {
                    respond(
                        id,
                        store
                            .messages()
                            .list_messages(&params)
                            .await
                            .map(|(messages, total)| MailResponse::MessageList { messages, total }),
                    )
                    .await
                }
                CrawlCommand::GetMessage(params) => {
                    match store.messages().get_message(&params).await {
                        Ok(Some(message)) => ok(
                            id,
                            serde_json::to_value(MailResponse::Message { message })
                                .unwrap_or_default(),
                        ),
                        Ok(None) => err_response(id, MailErrorCode::NotFound, "Message not found"),
                        Err(err) => err_response(id, MailErrorCode::DbError, &err.to_string()),
                    }
                }
                CrawlCommand::SearchMessages(params) => {
                    respond(
                        id,
                        store
                            .messages()
                            .search_messages(&params)
                            .await
                            .map(|messages| MailResponse::SearchResults { messages }),
                    )
                    .await
                }
                CrawlCommand::SendMessage(params) => match store.outbox().enqueue(&params).await {
                    Ok(queue_id) => ok(
                        id,
                        serde_json::to_value(MailResponse::SendQueued { queue_id })
                            .unwrap_or_default(),
                    ),
                    Err(err) => err_response(id, MailErrorCode::DbError, &err.to_string()),
                },
                CrawlCommand::MoveMessage(params) => {
                    match store
                        .messages()
                        .move_message(
                            &params.account_id,
                            params.uid,
                            &params.from_folder,
                            &params.to_folder,
                        )
                        .await
                    {
                        Ok(()) => {
                            let _ = event_tx.send(CrawlEvent::Mail(MailEvent::FlagsUpdated {
                                account_id: params.account_id,
                                uid: params.uid,
                                flags: vec![],
                            }));
                            ok(
                                id,
                                serde_json::to_value(MailResponse::Ok).unwrap_or_default(),
                            )
                        }
                        Err(err) => err_response(id, MailErrorCode::DbError, &err.to_string()),
                    }
                }
                CrawlCommand::CopyMessage(params) => {
                    match store
                        .messages()
                        .copy_message(&params.account_id, params.uid, &params.to_folder)
                        .await
                    {
                        Ok(()) => {
                            let _ = event_tx.send(CrawlEvent::Mail(MailEvent::NewMessages {
                                account_id: params.account_id,
                                folder: params.to_folder,
                                count: 1,
                            }));
                            ok(
                                id,
                                serde_json::to_value(MailResponse::Ok).unwrap_or_default(),
                            )
                        }
                        Err(err) => err_response(id, MailErrorCode::DbError, &err.to_string()),
                    }
                }
                CrawlCommand::DeleteMessage {
                    account_id,
                    folder,
                    uid,
                } => {
                    match store
                        .messages()
                        .delete_message(&account_id, &folder, uid)
                        .await
                    {
                        Ok(()) => {
                            let _ = event_tx.send(CrawlEvent::Mail(MailEvent::FlagsUpdated {
                                account_id,
                                uid,
                                flags: vec![crawl_ipc::types::MailFlag::Deleted],
                            }));
                            ok(
                                id,
                                serde_json::to_value(MailResponse::Ok).unwrap_or_default(),
                            )
                        }
                        Err(err) => err_response(id, MailErrorCode::DbError, &err.to_string()),
                    }
                }
                CrawlCommand::SetFlags(params) => {
                    match store
                        .messages()
                        .set_flags(
                            &params.account_id,
                            &params.folder,
                            params.uid,
                            &params.add,
                            &params.remove,
                        )
                        .await
                    {
                        Ok(flags) => {
                            let _ = event_tx.send(CrawlEvent::Mail(MailEvent::FlagsUpdated {
                                account_id: params.account_id,
                                uid: params.uid,
                                flags,
                            }));
                            ok(
                                id,
                                serde_json::to_value(MailResponse::Ok).unwrap_or_default(),
                            )
                        }
                        Err(err) => err_response(id, MailErrorCode::DbError, &err.to_string()),
                    }
                }
                CrawlCommand::SyncNow { account_id } => {
                    match accounts.sync_now(&account_id).await {
                        Ok(()) => ok(
                            id,
                            serde_json::to_value(MailResponse::SyncComplete { account_id })
                                .unwrap_or_default(),
                        ),
                        Err(err) => err_response(id, MailErrorCode::ImapError, &err.to_string()),
                    }
                }
                CrawlCommand::FetchBody { account_id, uid } => {
                    match handle_fetch_body(&store, &account_id, uid).await {
                        Ok(Some(message)) => ok(
                            id,
                            serde_json::to_value(MailResponse::Message { message })
                                .unwrap_or_default(),
                        ),
                        Ok(None) => err_response(
                            id,
                            MailErrorCode::NotFound,
                            "Message not found after fetch",
                        ),
                        Err(err) => err_response(id, MailErrorCode::ImapError, &err.to_string()),
                    }
                }
                CrawlCommand::SaveAttachment(payload) => {
                    match handle_save_attachment(&store, &payload).await {
                        Ok(()) => {
                            let _ = event_tx.send(CrawlEvent::Mail(MailEvent::AttachmentSaved {
                                account_id: payload.account_id,
                                uid: payload.uid,
                                attachment_id: payload.attachment_id,
                                dest_path: payload.dest_path,
                            }));
                            ok(
                                id,
                                serde_json::to_value(MailResponse::Ok).unwrap_or_default(),
                            )
                        }
                        Err(err) => err_response(id, MailErrorCode::DbError, &err.to_string()),
                    }
                }
                _ => CrawlResponse::error(
                    id,
                    error_code::METHOD_NOT_FOUND,
                    "Unsupported command for crawl-mail",
                ),
            }
        })
    });

    server.set_dispatcher(dispatcher);
    server.run().await?;
    Ok(())
}

async fn respond(
    id: Option<serde_json::Value>,
    result: Result<MailResponse, anyhow::Error>,
) -> CrawlResponse {
    match result {
        Ok(response) => ok(id, serde_json::to_value(response).unwrap_or_default()),
        Err(err) => err_response(id, MailErrorCode::DbError, &err.to_string()),
    }
}

fn ok(id: Option<serde_json::Value>, value: serde_json::Value) -> CrawlResponse {
    CrawlResponse::success(id, value)
}

fn err_response(
    id: Option<serde_json::Value>,
    code: MailErrorCode,
    message: &str,
) -> CrawlResponse {
    CrawlResponse::success(
        id,
        serde_json::to_value(MailResponse::Error {
            code,
            message: message.to_string(),
        })
        .unwrap_or_default(),
    )
}

async fn handle_fetch_body(
    store: &Store,
    account_id: &str,
    uid: u32,
) -> anyhow::Result<Option<MessageFull>> {
    let folder: Option<String> = store
        .messages()
        .find_folder_for_message(account_id, uid)
        .await?;

    let folder: String = match folder {
        Some(f) => f,
        None => return Ok(None),
    };

    let imap = store
        .accounts()
        .get_imap(account_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("No IMAP config"))?;

    let mut session =
        ImapSession::connect(&imap.host, imap.port, &imap.username, &imap.password).await?;
    session.select(&folder).await?;

    let raw_body = session.fetch_raw_body(uid).await?;
    let raw_body = match raw_body {
        Some(b) => b,
        None => return Ok(None),
    };

    let parsed = MessageParser::new()
        .parse(&raw_body)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse message body"))?;

    let body_text = parsed.body_text(0).map(|s| s.to_string());
    let body_html = parsed.body_html(0).map(|s| s.to_string());

    sqlx::query(
        "UPDATE messages SET body_text = ?, body_html = ?, body_fetched = 1
         WHERE account_id = ? AND uid = ?",
    )
    .bind(&body_text)
    .bind(&body_html)
    .bind(account_id)
    .bind(uid as i64)
    .execute(store.pool())
    .await?;

    let params = crawl_ipc::types::GetMessage {
        account_id: account_id.to_string(),
        uid,
        fetch_remote: false,
    };
    store.messages().get_message(&params).await
}

async fn handle_save_attachment(store: &Store, payload: &SaveAttachment) -> anyhow::Result<()> {
    let attachment = sqlx::query(
        "SELECT message_id, filename, part_index, cached_path FROM attachments WHERE id = ?",
    )
    .bind(&payload.attachment_id)
    .fetch_optional(store.pool())
    .await?;

    let (_, _, part_index, cached_path) = match attachment {
        Some(row) => (
            row.get::<String, _>("message_id"),
            row.get::<String, _>("filename"),
            row.get::<i64, _>("part_index"),
            row.get::<Option<String>, _>("cached_path"),
        ),
        None => anyhow::bail!("Attachment '{}' not found", payload.attachment_id),
    };

    if let Some(ref path) = cached_path {
        if std::path::Path::new(path).exists() {
            tokio::fs::copy(path, &payload.dest_path).await?;
            return Ok(());
        }
    }

    let folder: String = store
        .messages()
        .find_folder_for_message(&payload.account_id, payload.uid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Message not found"))?;

    let imap = store
        .accounts()
        .get_imap(&payload.account_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("No IMAP config"))?;

    let mut session =
        ImapSession::connect(&imap.host, imap.port, &imap.username, &imap.password).await?;
    session.select(&folder).await?;

    let raw_body = session
        .fetch_raw_body(payload.uid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("No body data for uid={}", payload.uid))?;

    let parsed = MessageParser::new()
        .parse(&raw_body)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse message body"))?;

    let attachments: Vec<_> = parsed.attachments().collect();
    let part = attachments
        .get(part_index as usize)
        .ok_or_else(|| anyhow::anyhow!("Attachment part index {} out of range", part_index))?;

    let data = part.contents();
    if let Some(parent) = std::path::Path::new(&payload.dest_path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&payload.dest_path, data).await?;

    sqlx::query("UPDATE attachments SET cached_path = ? WHERE id = ?")
        .bind(&payload.dest_path)
        .bind(&payload.attachment_id)
        .execute(store.pool())
        .await?;

    Ok(())
}
