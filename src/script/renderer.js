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
		e.addEventListener("click", () => {
			ipcRenderer.send("keyUpdate", parseInt(ev.target.getAttribute("data-n")), undefined);
		});
		e.addEventListener("contextmenu", () => {
			ipcRenderer.send("openPropertyInspector", parseInt(ev.target.getAttribute("data-n")));
		});
		ipcRenderer.send("keyUpdate", parseInt(ev.target.getAttribute("data-n")), dragging.id);
	} else if (ev.target.classList.contains("slider")) {
		if (!e.action.controllers.includes("Encoder")) return;
		e.addEventListener("click", () => {
			ipcRenderer.send("sliderUpdate", parseInt(ev.target.getAttribute("data-n")), undefined);
		});
		e.addEventListener("contextmenu", () => {
			ipcRenderer.send("openPropertyInspector", `s${ev.target.getAttribute("data-n")}`);
		});
		ipcRenderer.send("sliderUpdate", parseInt(ev.target.getAttribute("data-n")), dragging.id);
	} else {
		return;
	}
	
	ev.target.appendChild(e);
	
	ev.preventDefault();
}

for (const [index, action] of store.get("keys").entries()) {
	let div = document.querySelector(`div.key[data-n="${index}"]`);
	if (div == null) continue;
	
	if (action == undefined) continue;
	let image = createIcon(action);
	image.addEventListener("click", () => {
		image.remove();
		ipcRenderer.send("keyUpdate", index, undefined);
	});
	image.addEventListener("contextmenu", () => {
		ipcRenderer.send("openPropertyInspector", index);
	});
	div.appendChild(image);
	ipcRenderer.send("keyUpdate", index, action.uuid);
}
for (const [index, action] of store.get("sliders").entries()) {
	let div = document.querySelector(`div.slider[data-n="${index}"]`);
	
	if (action == undefined) continue;
	let image = createIcon(action);
	image.addEventListener("click", () => {
		image.remove();
		ipcRenderer.send("sliderUpdate", index, undefined);
	});
	image.addEventListener("contextmenu", () => {
		ipcRenderer.send("openPropertyInspector", `s${index}`);
	});
	div.appendChild(image);
	ipcRenderer.send("sliderUpdate", index, action.uuid);
}