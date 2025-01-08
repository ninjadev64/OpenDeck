use std::path::{Path, PathBuf};

use tiny_http::{Header, Response, Server};

fn mime(extension: &str) -> String {
	match extension {
		"htm" | "html" | "xhtml" => "text/html".to_owned(),
		"js" | "cjs" | "mjs" => "text/javascript".to_owned(),
		"css" => "text/css".to_owned(),
		"png" | "jpg" | "jpeg" | "gif" | "webp" => format!("image/{}", extension),
		"svg" => "image/svg+xml".to_owned(),
		_ => "application/octet-stream".to_owned(),
	}
}

/// Start a simple webserver to serve files of plugins that run in a browser environment.
pub async fn init_webserver(prefix: PathBuf) {
	let server = Server::http("0.0.0.0:57118").unwrap();
	for request in server.incoming_requests() {
		let mut url = urlencoding::decode(request.url()).unwrap().into_owned();
		if url.contains('?') {
			url = url.split_once('?').unwrap().0.to_owned();
		}
		#[cfg(target_os = "windows")]
		let url = url[1..].replace('/', "\\");

		// Ensure the requested path is within the OpenDeck config directory to prevent unrestricted access to the filesystem.
		let developer = match crate::store::Store::new("settings", &prefix, crate::store::Settings::default()) {
			Ok(store) => store.value.developer,
			Err(_) => false,
		};
		if !developer && !Path::new(&url).starts_with(&prefix) {
			let _ = request.respond(Response::empty(403));
			continue;
		}

		let access_control_allow_origin = Header {
			field: "Access-Control-Allow-Origin".parse().unwrap(),
			value: "*".parse().unwrap(),
		};

		// The Svelte frontend cannot call the connectElgatoStreamDeckSocket function on property inspector frames
		// because they are served from a different origin (this webserver on port 57118).
		// Instead, we have to inject a script onto all property inspector frames that receives a message
		// from the Svelte frontend over window.postMessage.

		// Additionally, Tauri cannot support window.open as seperate Tauri windows have seperate JavaScript contexts.
		// However, plugin property inspectors expect access to this function.
		// Instead, we have to inject a replacement window.open implementation that creates an IFrame element
		// and requests the Svelte frontend to maximise the property inspector.

		if url.ends_with("|opendeck_property_inspector") {
			let path = &url[..url.len() - 28];
			if !matches!(tokio::fs::try_exists(path).await, Ok(true)) {
				let _ = request.respond(Response::empty(404).with_header(access_control_allow_origin));
				continue;
			}

			let mut content = tokio::fs::read_to_string(path).await.unwrap_or_default();
			content += r#"
				<div id="opendeck_iframe_container" style="position: absolute; z-index: 100; top: 0; left: 0; width: 100%; height: 100%; display: none;" />
				<script>
					const opendeck_window_open = window.open;
					const opendeck_iframe_container = document.getElementById("opendeck_iframe_container");

					window.addEventListener("message", ({ data }) => {
						if (data.event == "connect") {
							connectElgatoStreamDeckSocket(...data.payload);
						} else if (data.event == "windowClosed") {
							opendeck_iframe_container.innerHtml = "";
							opendeck_iframe_container.style.display = "none";
						}
					});

					window.open = (url, target) => {
						if (target && !(target == "_self" || target == "_top")) {
							top.postMessage({ event: "openUrl", payload: url.startsWith("http") ? url : new URL(url, window.location.href).href }, "*");
							return;
						}
						let iframe = document.createElement("iframe");
						iframe.src = url;
						iframe.style.flexGrow = "1";
						iframe.onload = () => {
							iframe.contentWindow.opener = window;
							iframe.contentWindow.onbeforeunload = () => top.postMessage({ event: "windowClosed", payload: window.name }, "*");
							iframe.contentWindow.document.body.style.overflowY = "auto";
						};
						opendeck_iframe_container.appendChild(iframe);
						opendeck_iframe_container.style.display = "flex";
						top.postMessage({ event: "windowOpened", payload: window.name }, "*");
						return iframe.contentWindow;
					};
				</script>
			"#;

			let mut response = Response::from_string(content);
			response.add_header(access_control_allow_origin);
			response.add_header(Header {
				field: "Content-Type".parse().unwrap(),
				value: "text/html".parse().unwrap(),
			});
			let _ = request.respond(response);
		} else {
			if !matches!(tokio::fs::try_exists(&url).await, Ok(true)) {
				let _ = request.respond(Response::empty(404).with_header(access_control_allow_origin));
				continue;
			}

			let mime_type = mime(&match Path::new(&url).extension() {
				Some(extension) => extension.to_string_lossy().into_owned(),
				None => "html".to_owned(),
			});

			let content_type = Header {
				field: "Content-Type".parse().unwrap(),
				value: mime_type.parse().unwrap(),
			};

			if mime_type.starts_with("text/") || mime_type == "image/svg+xml" {
				let mut response = Response::from_string(tokio::fs::read_to_string(url).await.unwrap_or_default());
				response.add_header(access_control_allow_origin);
				response.add_header(content_type);
				let _ = request.respond(response);
			} else {
				let mut response = Response::from_file(match tokio::fs::File::open(url).await {
					Ok(file) => file.into_std().await,
					Err(_) => continue,
				});
				response.add_header(access_control_allow_origin);
				response.add_header(content_type);
				let _ = request.respond(response);
			}
		}
	}
}
