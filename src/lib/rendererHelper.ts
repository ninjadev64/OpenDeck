import type { ActionState } from "./ActionState";

export function getImage(image: string | undefined, fallback: string): string {
	if (!image) return fallback;
	if (!image.startsWith("data:")) return "http://localhost:57118" + image;
	const svgxmlre = /^data:image\/svg\+xml,(.+)/;
	const base64re = /^data:image\/(apng|avif|gif|jpeg|png|svg\+xml|webp|bmp|x-icon|tiff);base64,([A-Za-z0-9+/]+={0,2})?/;
	if (svgxmlre.test(image)) {
		image = "data:image/svg+xml;base64," + btoa(decodeURIComponent((svgxmlre.exec(image) as RegExpExecArray)[1].replace(/\;$/, "")));
	}
	if (base64re.test(image)) {
		let exec = base64re.exec(image)!;
		if (!exec[2]) return fallback;
		else image = exec[0];
	}
	return image;
}

export default async function renderImage(state: ActionState) {
	// Create canvas
	let canvas = document.createElement("canvas");
	canvas.width = 144;
	canvas.height = 144;
	let context = canvas.getContext("2d");
	if (!context) return;

	// White background
	context.fillStyle = "white";
	context.fillRect(0, 0, canvas.width, canvas.height);

	// Load image
	let image = document.createElement("img");
	image.crossOrigin = "anonymous";
	image.src = getImage(state.image, undefined!);
	if (image.src == undefined) return;
	await new Promise((resolve) => {
		image.onload = resolve;
	});

	// Draw image
	context.drawImage(image, 0, 0, canvas.width, canvas.height);

	// Draw text
	if (state.show) {
		context.textAlign = "center";
		context.font = `${state.size}px serif`;
		context.fillStyle = state.colour;
		let x = canvas.width / 2;
		let y = canvas.height / 2 + (parseInt(state.size) / 4);
		switch (state.alignment) {
			case "top": y = parseInt(state.size); break;
			case "bottom": y = canvas.height - 5; break;
		}
		context.fillText(state.text, x, y);
		if (state.underline) {
			let width = context.measureText(state.text).width;
			context.fillRect(x - (width / 2), y + 2, width, 3);
		}
	}

	console.log(canvas.toDataURL("image/jpeg"));
	canvas.remove();
}
