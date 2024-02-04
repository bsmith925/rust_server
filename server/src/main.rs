use axum::{
  routing::get,
  Router,
  extract::Path as AxumPath,
  http::{StatusCode, Response},
  response::IntoResponse,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() {
  let app = Router::new()
      .route("/images/:image_name", get(serve_image))
      .route("/images/thumbnails/:image_name", get(serve_thumbnail)) // Add this line
      .route("/images", get(serve_index));

  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  println!("Listening on {}", addr);
  axum_server::bind(addr)
      .serve(app.into_make_service())
      .await
      .unwrap();
}

async fn serve_index() -> impl IntoResponse {
  let image_dir = Path::new("/Users/bsmith/Personal/images");
  if let Err(e) = generate_thumbnails(image_dir) {
      eprintln!("Failed to generate thumbnails: {}", e);
      return (
          StatusCode::INTERNAL_SERVER_ERROR,
          "Failed to generate thumbnails".to_string(),
      ).into_response();
  }
  match generate_image_list_html(image_dir) {
      Ok(html) => Response::builder()
          .status(StatusCode::OK)
          .header("Content-Type", "text/html; charset=utf-8")
          .body(html.into())
          .unwrap(),
      Err(_) => (
          StatusCode::INTERNAL_SERVER_ERROR,
          "Failed to generate image list".to_string(),
      ).into_response(),
  }
}


async fn serve_image(AxumPath(image_name): AxumPath<String>) -> impl IntoResponse {
  let base_path = PathBuf::from("/Users/bsmith/Personal/images");
  let file_path = base_path.join(&image_name);


  match fs::read(file_path) {
      Ok(data) => {
          let content_type = match image_name.rsplit('.').next() {
              Some("png") => "image/png",
              Some("jpg") | Some("jpeg") => "image/jpeg",
              Some("gif") => "image/gif",
              // Add more image formats as needed
              _ => "application/octet-stream", // Default to binary data
          };
          Response::builder()
              .status(StatusCode::OK)
              .header("Content-Type", content_type)
              .body(data.into())
              .unwrap()
      },
      Err(_) => (
          StatusCode::NOT_FOUND,
          "Image not found".to_string(),
      ).into_response(),
  }
}

async fn serve_thumbnail(AxumPath(image_name): AxumPath<String>) -> impl IntoResponse {
  let base_path = PathBuf::from("/Users/bsmith/Personal/images/thumbnails");
  let file_path = base_path.join(&image_name);

  match fs::read(file_path) {
    Ok(data) => {
        let content_type = match image_name.rsplit('.').next() {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            // Add more image formats as needed
            _ => "application/octet-stream", // Default to binary data
        };
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", content_type)
            .body(data.into())
            .unwrap()
    },
    Err(_) => (
        StatusCode::NOT_FOUND,
        "Image not found".to_string(),
    ).into_response(),
}
}





fn generate_image_list_html(image_dir: &Path) -> Result<String, std::io::Error> {
  let thumbnail_dir = image_dir.join("thumbnails");
  let paths = fs::read_dir(thumbnail_dir)?;
  let mut image_links = String::new();

  for path in paths {
      let file_name = path?.file_name().into_string().unwrap();
      let orig_file_name = file_name.trim_start_matches("thumb_");
      let display_name = orig_file_name.split('.').next().unwrap_or(""); // Remove the extension for display
      image_links.push_str(&format!(
          "<div class=\"gallery-item\"><a href=\"/images/{}\"><img src=\"/images/thumbnails/{}\" alt=\"{}\" /><span>{}</span></a></div>\n",
          orig_file_name, file_name, orig_file_name, display_name
      ));
  }

  let html_template = fs::read_to_string("template.html")?;
  Ok(html_template.replace("{{image_links}}", &image_links))
}











// Generate thumbnails on server start
fn generate_thumbnails(image_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
  let thumbnail_dir = image_dir.join("thumbnails");
  fs::create_dir_all(&thumbnail_dir)?;

  for entry in fs::read_dir(image_dir)? {
    let path = entry?.path();
    if path.is_file() {
      let thumb_path = thumbnail_dir.join(format!("thumb_{}", path.file_name().unwrap().to_str().unwrap()));
      if !thumb_path.exists() {
        create_thumbnail(&path, &thumb_path)?;
      }
    }
  }

  Ok(())
}

fn create_thumbnail(original: &Path, thumb_path: &Path) -> Result<(), image::ImageError> {
  let img = image::open(original)?;
  let thumbnail = img.thumbnail(100, 100); // Adjust size as needed
  thumbnail.save(thumb_path)?;
  Ok(())
}