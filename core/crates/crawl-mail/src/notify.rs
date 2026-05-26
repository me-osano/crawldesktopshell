use zbus::Connection;

pub async fn notify_new_mail(summary: &str) -> anyhow::Result<()> {
    let conn = Connection::session().await?;

    let proxy = zbus::proxy::Proxy::new(
        &conn,
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
    )
    .await?;

    let _ = proxy
        .call_method(
            "Notify",
            &(
                "crawl-mail",
                0u32,
                "",
                summary,
                "",
                Vec::<String>::new(),
                std::collections::HashMap::<&str, zbus::zvariant::Value<'_>>::new(),
                -1i32,
            ),
        )
        .await?;

    Ok(())
}
