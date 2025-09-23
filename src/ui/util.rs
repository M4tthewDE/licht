pub fn humanize_runtime(runtime: u64) -> String {
    let hours = runtime / 60;
    let minutes = runtime % 60;

    if hours == 0 {
        format!("{minutes}m")
    } else {
        format!("{hours}h{minutes}m")
    }
}

pub fn build_poster_url(poster_path: Option<String>) -> String {
    if let Some(poster_path) = poster_path {
        format!(
            "https://image.tmdb.org/t/p/w600_and_h900_bestv2{}",
            poster_path
        )
    } else {
        "https://www.themoviedb.org/assets/2/v4/glyphicons/basic/glyphicons-basic-38-picture-grey-c2ebdbb057f2a7614185931650f8cee23fa137b93812ccb132b9df511df1cfac.svg".to_string()
    }
}
