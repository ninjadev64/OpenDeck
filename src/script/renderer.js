const { ipcRenderer } = require("electron");
const store = require("../script/store");

let actionList = document.getElementById("action-list");

let dragging;
document.addEventListener("dragend", () => { dragging = undefined; });

function createIcon(action) {
	let image = document.createElement("img");
	image.src = action.icon;
	image.id = action.uuid;
	image.alt = action.tooltip;
	image.className = "icon";
	image.draggable = true;
	image.action = action;
	image.addEventListener("dragstart", () => { dragging = image; });
	return image;
}

ipcRenderer.send("requestCategories");
ipcRenderer.on("categories", (_, categories) => {
	for (const [category, actions] of Object.entries(categories)) {
		let heading = document.createElement("h3");
		heading.innerText = category;
		actionList.appendChild(heading);
		
		actions.forEach((action) => {
			let div = document.createElement("div");
			div.className = "action";

			let image = createIcon(action);

			let span = document.createElement("span");
			span.innerText = action.name;

			div.appendChild(image);
			div.appendChild(span);
			actionList.appendChild(div);
		});
	}

	let pluginStore = document.createElement("button");
	pluginStore.innerText = "Open plugin store";
	pluginStore.addEventListener("click", () => {
		window.open("pluginstore.html", undefined, "nodeIntegration=yes,contextIsolation=no,autoHideMenuBar=yes");
	});
	pluginStore.style = "position: absolute; bottom: 10px;";
	actionList.append(pluginStore);

	let settings = document.createElement("button");
	settings.innerText = "Open settings";
	settings.addEventListener("click", () => {
		window.open("settings.html", undefined, "nodeIntegration=yes,contextIsolation=no");
	});
	settings.style = "position: absolute; bottom: 10px; right: 30px;";
	actionList.append(settings);
});

Array.from(document.getElementsByClassName("key")).forEach((div) => {
	div.addEventListener("dragover", (ev) => { dragover(ev); });
	div.addEventListener("drop", (ev) => { drop(ev); });
});
Array.from(document.getElementsByClassName("slider")).forEach((div) => {
	div.addEventListener("dragover", (ev) => { dragover(ev); });
	div.addEventListener("drop", (ev) => { drop(ev); });
});

function dragover(ev) {
	let e = dragging;
	if (ev.target.children.length != 0) return;
	if (ev.target.classList.contains("key")) {
		if (!e.action.controllers.includes("Keypad")) return;
	} else if (ev.target.classList.contains("slider")) {
		if (!e.action.controllers.includes("Encoder")) return;
	} else {
		return;
	}
	ev.preventDefault();
}

function drop(ev) {
	if (ev.target.children.length != 0) return;

	let e = dragging.cloneNode();
	e.action = dragging.action;
	e.addEventListener("click", (e) => {
		e.target.remove();
	});

	if (ev.target.classList.contains("key")) {
		if (!e.action.controllers.includes("Keypad")) return;

		ipcRenderer.send("createInstance", e.action.uuid, parseInt(ev.target.id), "Keypad");
		ipcRenderer.once("instanceCreated", (_event, instance) => {
			e.instance = instance;
			e.addEventListener("click", () => {
				ipcRenderer.send("keyUpdate", instance.context, undefined);
			});
			e.addEventListener("contextmenu", () => {
				ipcRenderer.send("openPropertyInspector", instance.context);
			});
			ev.target.appendChild(e);
		});
	} else if (ev.target.classList.contains("slider")) {
		if (!e.action.controllers.includes("Encoder")) return;

		ipcRenderer.send("createInstance", e.action.uuid, ev.target.id, "Encoder");
		ipcRenderer.once("instanceCreated", (_event, instance) => {
			e.instance = instance;
			e.addEventListener("click", () => {
				ipcRenderer.send("sliderUpdate", instance.context, undefined);
			});
			e.addEventListener("contextmenu", () => {
				ipcRenderer.send("openPropertyInspector", instance.context);
			});
			ev.target.appendChild(e);
		});
	} else {
		return;
	}
	
	ev.preventDefault();
}

store.get("keys").forEach((instance) => {
	if (instance == undefined) return;

	let div = document.getElementById(instance.context.toString());
	if (div == null) return;
	
	let action = instance.action;
	let image = createIcon(action);
	image.instance = instance;
	image.addEventListener("click", () => {
		image.remove();
		ipcRenderer.send("keyUpdate", instance.context, undefined);
	});
	image.addEventListener("contextmenu", () => {
		ipcRenderer.send("openPropertyInspector", instance.context);
	});
	div.appendChild(image);
	ipcRenderer.send("keyUpdate", instance.context, instance);
});
store.get("sliders").forEach((instance) => {
	if (instance == undefined) return;

	let div = document.getElementById(instance.context);
	
	let action = instance.action;
	let image = createIcon(action);
	image.instance = instance;
	image.addEventListener("click", () => {
		image.remove();
		ipcRenderer.send("sliderUpdate", instance.context, undefined);
	});
	image.addEventListener("contextmenu", () => {
		ipcRenderer.send("openPropertyInspector", instance.context);
	});
	div.appendChild(image);
	ipcRenderer.send("sliderUpdate", instance.context, instance);
});

function flash(key, image) {
	let div = document.getElementById(key.toString());
	let img = document.createElement("img");
	img.src = image;
	img.classList.add("flash");
	div.appendChild(img);
	setTimeout(() => img.style.opacity = "0", 1000);
	setTimeout(() => img.remove(), 2500);
}

ipcRenderer.on("showAlert", (_event, context) => {
	flash(context, "../assets/alert.png");
});

ipcRenderer.on("showOk", (_event, context) => {
	flash(context, "../assets/check.png");
});