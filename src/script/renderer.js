const { ipcRenderer } = require("electron");

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

function updateState(instance) {
	let div = document.getElementById(instance.context.toString());
	if (!div) {
		if (instance.device != selectedDevice) return;
		div = document.createElement("div");
		div.id = instance.context.toString();
		div.className = "instance";
		div.instance = instance;
		div.addEventListener("click", () => {
			ipcRenderer.send("slotUpdate", instance.context, undefined);
		});
		div.addEventListener("contextmenu", () => {
			ipcRenderer.send("openPropertyInspector", instance.context);
		});
		document.getElementById(instance.type + instance.position).appendChild(div);
	}
	div.textContent = "";
	let state = instance.states[instance.state];
	let image = document.createElement("img");
	image.src = state.image;
	image.alt = instance.action.tooltip;
	image.className = "icon";
	div.appendChild(image);
	let title = document.createElement("span");
	title.innerText = state.title;
	title.style.position = "absolute";
	title.style.left = "50%";
	switch (state.titleAlignment) {
		case "top": title.style.top = "0"; break;
		case "middle": title.style.top = "50%"; break;
		case "bottom": title.style.top = "100%"; break;
	}
	title.style.transform = "translate(-50%, -50%)";
	title.style.opacity = state.showTitle ? 1 : 0;
	title.style.color = state.titleColour;
	title.style.fontWeight = state.titleFontStyle.toLowerCase().includes("bold") ? "bold" : "normal";
	title.style.fontStyle = state.titleFontStyle.toLowerCase().includes("italic") ? "italic" : "normal";
	title.style.fontSize = state.titleFontSize + "px";
	title.style.textDecorationLine = state.titleUnderline ? "underline" : "none";
	div.appendChild(title);
}

ipcRenderer.on("categories", (_, categories) => {
	for (const [category, actions] of Object.entries(categories)) {
		let heading = document.createElement("h3");
		heading.innerText = category;
		actionList.appendChild(heading);
		
		actions.forEach((action) => {
			if (!action.visibleInActionsList) return;
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
	pluginStore.innerText = "Manage plugins";
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
	settings.style = "position: absolute; bottom: 10px; right: 45px;";
	actionList.append(settings);
});
ipcRenderer.send("requestCategories");

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

	if (ev.target.classList.contains("key")) {
		if (!dragging.action.controllers.includes("Keypad")) return;
		ipcRenderer.send("createInstance", dragging.action.uuid, selectedDevice, "key", parseInt(ev.target.id.slice(-1)), 0);
	} else if (ev.target.classList.contains("slider")) {
		if (!dragging.action.controllers.includes("Encoder")) return;
		ipcRenderer.send("createInstance", dragging.action.uuid, selectedDevice, "slider", parseInt(ev.target.id.slice(-1)), 0);
	} else {
		return;
	}
	
	ev.preventDefault();
}

function populateSlots(device) {
	let slots = document.getElementById("slots");
	slots.textContent = "";
	let keyIndex = 0;

	for (let r = 0; r < device.rows; r++) {
		let tr = document.createElement("tr");
		if (keyIndex == 0) {
			for (let s = 0; s < device.sliders; s++) {
				let td = document.createElement("td");
				td.rowSpan = device.rows;
				let div = document.createElement("div");
				div.classList.add("slider");
				div.id = `slider${s}`;
				div.addEventListener("dragover", (ev) => dragover(ev));
				div.addEventListener("drop", (ev) => drop(ev));
				td.appendChild(div);
				tr.appendChild(td);
			}
		}
		for (let c = 0; c < device.columns; c++) {
			let td = document.createElement("td");
			let div = document.createElement("div");
			div.classList.add("key");
			div.id = `key${keyIndex}`;
			div.addEventListener("dragover", (ev) => dragover(ev));
			div.addEventListener("drop", (ev) => drop(ev));
			td.appendChild(div);
			tr.appendChild(td);
			keyIndex++;
		}
		slots.appendChild(tr);
	}
}

let devices;
let selectedDevice;
let deviceSelect = document.getElementById("device");
deviceSelect.addEventListener("change", () => {
	selectedDevice = deviceSelect.value;
	ipcRenderer.send("requestProfiles", selectedDevice);
	populateSlots(devices[selectedDevice]);
});
ipcRenderer.on("devices", (_event, d) => {
	devices = d;
	selectedDevice = Object.keys(devices)[0];
	populateSlots(devices[selectedDevice]);
	for (const [id, device] of Object.entries(devices)) {
		let o = document.createElement("option");
		o.value = id;
		o.innerText = device.name;
		deviceSelect.appendChild(o);
	}
	ipcRenderer.send("requestProfiles", selectedDevice);
});
ipcRenderer.send("requestDevices");

let selectedProfile;
let profileManager;
let profileSelect = document.getElementById("profile");
profileSelect.addEventListener("change", () => {
	if (profileSelect.value == "manager") {
		if (!profileManager || profileManager.closed) {
			profileManager = window.open("profiles.html", undefined, "nodeIntegration=yes,contextIsolation=no,autoHideMenuBar=yes,alwaysOnTop=yes");
		} else {
			profileManager.focus();
		}
		profileSelect.value = selectedProfile;
	} else {
		ipcRenderer.send("profileUpdate", selectedDevice, profileSelect.value);
	}
});
ipcRenderer.on("profiles", (_event, profiles, selected) => {
	selectedProfile = selected;
	Array.from(document.getElementsByClassName("instance")).forEach((e) => e.remove());
	[].concat(profiles[selectedProfile].key, profiles[selectedProfile].slider).forEach((position) => { position.forEach((instance) => {
		if (!instance) return;
		ipcRenderer.send("slotUpdate", instance.context, instance);
	})});

	profileSelect.textContent = "";
	for (const [id, profile] of Object.entries(profiles)) {
		let o = document.createElement("option");
		o.value = id;
		o.innerText = profile.name;
		if (id == selectedProfile) o.selected = true;
		profileSelect.appendChild(o);
	}
	let manager = document.createElement("option");
	manager.value = "manager";
	manager.innerText = "Manage profiles...";
	profileSelect.appendChild(manager);
});

ipcRenderer.on("updateState", (_event, context, instance) => {
	if (!instance) document.getElementById(context).remove();
	else updateState(instance);
});

function flash(context, image) {
	context = context.split(".");
	let div = document.getElementById(context[1] + context[2]);
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