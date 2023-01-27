const { ipcRenderer } = require("electron");
const store = require("../script/store");

let actionList = document.getElementById("action-list");

ipcRenderer.on("categories", (_, categories) => {
	for (const [category, actions] of Object.entries(categories)) {
		let heading = document.createElement("h3");
		heading.innerText = category;
		actionList.appendChild(heading);
		
		actions.forEach((action) => {
			let div = document.createElement("div");
			div.className = "action";

			let image = document.createElement("img");
			image.src = action.icon;
			image.id = action.uuid;
			image.alt = action.tooltip;
			image.className = "icon";
			image.draggable = true;
			image.addEventListener("dragstart", (ev) => { ev.dataTransfer.setData("text", action.uuid); });

			let span = document.createElement("span");
			span.innerText = action.name;

			div.appendChild(image);
			div.appendChild(span);
			actionList.appendChild(div);
		});
	}
	let settings = document.createElement("button");
	settings.innerText = "Open settings";
	settings.addEventListener("click", () => {
		window.open("settings.html", undefined, "nodeIntegration=yes,contextIsolation=no");
	});
	settings.style = "position: absolute; bottom: 10px;";
	actionList.append(settings);
});

Array.from(document.getElementsByClassName("key")).forEach((div) => {
	div.addEventListener("dragover", (ev) => { ev.preventDefault(); });
	div.addEventListener("drop", (ev) => { drop(ev); });
});

function drop(ev) {
	if (!ev.target.classList.contains("key")) return;
	if (ev.target.children.length == 0) {
		ev.preventDefault();
		let e = document.getElementById(ev.dataTransfer.getData("text")).cloneNode();
		e.addEventListener("click", (eve) => {
			eve.target.remove();
			ipcRenderer.send("keyUpdate", parseInt(ev.target.id), undefined);
		});
		ev.target.appendChild(e);
		ipcRenderer.send("keyUpdate", parseInt(ev.target.id), ev.dataTransfer.getData("text"));
	}
}

for (const [index, action] of store.get("keys").entries()) {
	let div = document.getElementById(`${index}`);
	if (div == null) continue;
	
	if (action == undefined) continue;
	let image = document.createElement("img");
	image.src = action.icon;
	image.id = action.uuid;
	image.alt = action.tooltip;
	image.className = "icon";
	image.draggable = true;
	image.addEventListener("dragstart", (ev) => { ev.dataTransfer.setData("text", action.uuid); });
	image.addEventListener("click", () => {
		image.remove();
		ipcRenderer.send("keyUpdate", index, undefined);
	});

	div.appendChild(image);
}