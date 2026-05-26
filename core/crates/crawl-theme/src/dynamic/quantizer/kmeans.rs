use crate::dynamic::hct::lab::{lab_distance, rgb_to_lab};

pub type RgbCluster = ((u8, u8, u8), (u8, u8, u8), usize);

pub fn kmeans_cluster(
    pixels: &[(u8, u8, u8)],
    k: usize,
    iterations: usize,
) -> Vec<RgbCluster> {
    if pixels.is_empty() || k == 0 {
        return Vec::new();
    }

    if pixels.len() < k {
        let mut seen: Vec<(u8, u8, u8)> = Vec::new();
        for &p in pixels {
            if !seen.contains(&p) {
                seen.push(p);
            }
        }
        let mut result: Vec<RgbCluster> = seen
            .into_iter()
            .map(|c| {
                let count = pixels.iter().filter(|&&p| p == c).count();
                (c, c, count)
            })
            .collect();
        result.sort_by(|a, b| b.2.cmp(&a.2));
        result.truncate(k);
        return result;
    }

    let colors_lab: Vec<(f32, f32, f32)> = pixels.iter().map(|&(r, g, b)| rgb_to_lab(r, g, b)).collect();

    let mut sorted_indices: Vec<usize> = (0..colors_lab.len()).collect();
    sorted_indices.sort_by(|&a, &b| colors_lab[a].0.partial_cmp(&colors_lab[b].0).unwrap());

    let step = sorted_indices.len() / k;
    let mut centroids: Vec<(f32, f32, f32)> = (0..k)
        .map(|i| colors_lab[sorted_indices[i * step]])
        .collect();

    let mut assignments = vec![0usize; colors_lab.len()];

    for _ in 0..iterations {
        for (idx, &color) in colors_lab.iter().enumerate() {
            let mut min_dist = f32::MAX;
            let mut min_cluster = 0;
            for (i, &centroid) in centroids.iter().enumerate() {
                let dist = lab_distance(color, centroid);
                if dist < min_dist {
                    min_dist = dist;
                    min_cluster = i;
                }
            }
            assignments[idx] = min_cluster;
        }

        let mut new_centroids: Vec<(f32, f32, f32)> = Vec::with_capacity(k);
        for i in 0..k {
            let mut cluster_l = Vec::new();
            let mut cluster_a = Vec::new();
            let mut cluster_b = Vec::new();
            for (j, &cidx) in assignments.iter().enumerate() {
                if cidx == i {
                    let (l, a, b) = colors_lab[j];
                    cluster_l.push(l);
                    cluster_a.push(a);
                    cluster_b.push(b);
                }
            }
            if cluster_l.is_empty() {
                new_centroids.push(centroids[i]);
            } else {
                let avg_l = cluster_l.iter().sum::<f32>() / cluster_l.len() as f32;
                let avg_a = cluster_a.iter().sum::<f32>() / cluster_a.len() as f32;
                let avg_b = cluster_b.iter().sum::<f32>() / cluster_b.len() as f32;
                new_centroids.push((avg_l, avg_a, avg_b));
            }
        }
        centroids = new_centroids;
    }

    let mut cluster_counts = vec![0usize; k];
    let mut cluster_reps: Vec<((u8, u8, u8), f32)> = vec![(pixels[0], f32::MAX); k];

    for (idx, &color_lab) in colors_lab.iter().enumerate() {
        let cidx = assignments[idx];
        cluster_counts[cidx] += 1;

        let dist = lab_distance(color_lab, centroids[cidx]);
        if dist < cluster_reps[cidx].1 {
            cluster_reps[cidx] = (pixels[idx], dist);
        }
    }

    let mut results: Vec<RgbCluster> = Vec::new();
    for i in 0..k {
        if cluster_counts[i] > 0 {
            let (cr, cg, cb) = crate::dynamic::hct::lab::lab_to_rgb(
                centroids[i].0, centroids[i].1, centroids[i].2,
            );
            let centroid_rgb = (cr, cg, cb);
            let rep_rgb = cluster_reps[i].0;
            results.push((centroid_rgb, rep_rgb, cluster_counts[i]));
        }
    }

    results.sort_by(|a, b| b.2.cmp(&a.2));
    results
}
