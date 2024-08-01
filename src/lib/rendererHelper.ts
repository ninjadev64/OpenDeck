import { invoke } from "@tauri-apps/api";

import type { ActionState } from "./ActionState";
import type { Context } from "./Context";

export function getImage(image: string | undefined, fallback: string | undefined): string {
	if (!image) return fallback ? getImage(fallback, undefined) : "/alert.png";
	if (!image.startsWith("data:")) return "http://localhost:57118/" + image;
	const svgxmlre = /^data:image\/svg\+xml,(.+)/;
	const base64re = /^data:image\/(apng|avif|gif|jpeg|png|svg\+xml|webp|bmp|x-icon|tiff);base64,([A-Za-z0-9+/]+={0,2})?/;
	if (svgxmlre.test(image)) {
		image = "data:image/svg+xml;base64," + btoa(decodeURIComponent((svgxmlre.exec(image) as RegExpExecArray)[1].replace(/\;$/, "")));
	}
	if (base64re.test(image)) {
		let exec = base64re.exec(image)!;
		if (!exec[2]) return fallback ? getImage(fallback, undefined) : "/alert.png";
		else image = exec[0];
	}
	return image;
}

export async function renderImage(canvas: HTMLCanvasElement, slotContext: Context, state: ActionState, fallback: string | undefined, showOk: boolean, showAlert: boolean, processImage: boolean, active: boolean, pressed: boolean) {
	// Create canvas
	let scale = 1;
	if (!canvas) {
		canvas = document.createElement("canvas");
		canvas.width = 144;
		canvas.height = 144;
	} else {
		scale = canvas.width / 144;
	}

	let context = canvas.getContext("2d");
	if (!context) return;
	context.clearRect(0, 0, canvas.width, canvas.height);

	// Load image
	let image = document.createElement("img");
	image.crossOrigin = "anonymous";
	image.src = processImage ? getImage(state.image, fallback) : state.image;
	if (image.src == undefined) return;
	await new Promise((resolve) => {
		image.onload = resolve;
	});

	// Draw image
	context.imageSmoothingQuality = "high";
	context.drawImage(image, 0, 0, canvas.width, canvas.height);

	// Draw text
	if (state.show) {
		const size = parseInt(state.size) * 2 * scale;
		context.textAlign = "center";
		context.font =
			(state.style.includes("Bold") ? "bold " : "") + (state.style.includes("Italic") ? "italic " : "") +
			`${size}px "${state.family}", sans-serif`;
		context.fillStyle = state.colour;
		context.strokeStyle = "black";
		context.lineWidth = 3 * scale;
		context.textBaseline = "top";
		let x = canvas.width / 2;
		let y = canvas.height / 2 - (size * state.text.split("\n").length * 0.5);
		switch (state.alignment) {
			case "top": y = -(size * 0.2); break;
			case "bottom": y = canvas.height - (size * state.text.split("\n").length) - context.lineWidth; break;
		}
		for (const [ index, line ] of Object.entries(state.text.split("\n"))) {
			context.strokeText(line, x, y + (size * parseInt(index)));
			context.fillText(line, x, y + (size * parseInt(index)));
			if (state.underline) {
				let width = context.measureText(line).width;
				// Set to black for the outline, since it uses the same fill style info as the text colour.
				context.fillStyle = "black";
				context.fillRect(x - (width / 2) - 3, y + (size * parseInt(index)) + size, width + 6, 9);
				// Reset to the user's choice of text colour.
				context.fillStyle = state.colour;
				context.fillRect(x - (width / 2), y + (size * parseInt(index)) + size + 4, width, 3);
			}
		}
	}

	if (showOk) {
		let okImage = document.createElement("img");
		okImage.crossOrigin = "anonymous";
		okImage.src = "/ok.png";
		await new Promise((resolve) => {
			okImage.onload = resolve;
		});
		context.drawImage(okImage, 0, 0, canvas.width, canvas.height);
	}

	if (showAlert) {
		let alertImage = document.createElement("img");
		alertImage.crossOrigin = "anonymous";
		alertImage.src = "/alert.png";
		await new Promise((resolve) => {
			alertImage.onload = resolve;
		});
		context.drawImage(alertImage, 0, 0, canvas.width, canvas.height);
	}

	
	//make the image smaller while the button is pressed
	if (pressed) {
		let smallCanvas = document.createElement("canvas");
		smallCanvas.width = canvas.width;
		smallCanvas.height = canvas.height;
		let Newcontext = smallCanvas.getContext("2d");
		let margin = .1;
		if (Newcontext) {
			Newcontext.drawImage(canvas, canvas.width*margin, canvas.height*margin, canvas.width-canvas.height*margin*2, canvas.height-canvas.height*margin*2);
			canvas = smallCanvas;
		}
	}

	if (active) setTimeout(async () => await invoke("update_image", { context: slotContext, image: canvas.toDataURL("image/jpeg") }), 10);
}
