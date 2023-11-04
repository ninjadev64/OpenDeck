const { ipcRenderer } = require("electron");
const store = require("../../build/store").default;

let list = document.getElementById("profiles");

let selectedDevice = Object.keys(store.get("devices"))[0];
let deviceSelect = document.getElementById("device-selector");
deviceSelect.addEventListener("change", () => {
	selectedDevice = deviceSelect.value;
	updateList(store.get("devices")[selectedDevice].profiles);
});
for (const [ id, device ] of Object.entries(store.get("devices"))) {
	let o = document.createElement("option");
	o.value = id;
	o.innerText = device.name;
	deviceSelect.appendChild(o);
}

document.getElementById("create").addEventListener("click", () => {
	let id = Date.now().toString(36) + Math.random().toString(36).substring(2);
	let name = document.getElementById("name").value;
	updateList({ ...(store.get("devices")[selectedDevice].profiles), [id]: { name } });
	ipcRenderer.send("createProfile", selectedDevice, name, id);
	document.getElementById("name").value = "";
});

function updateList(profiles) {
	list.textContent = "";
	for (const [ id, profile ] of Object.entries(profiles)) {
		let t = document.createElement("li");
		t.innerText = profile.name;
		
		let i = document.createElement("img");
		i.src = "../assets/cross.png";
		i.className = "deleteProfile";
		i.addEventListener("click", () => {
			if (Object.keys(store.get("devices")[selectedDevice].profiles).length < 2) return;
			store.delete("devices." + selectedDevice + ".profiles." + id);
			if (store.get("devices")[selectedDevice].selectedProfile == id) {
				ipcRenderer.send("profileUpdate", selectedDevice, Object.keys(store.get("devices")[selectedDevice].profiles)[0]);
			} else {
				ipcRenderer.send("profileUpdate", selectedDevice, store.get("devices")[selectedDevice].selectedProfile);
			}
			t.remove();
		});
		t.appendChild(i);
		
		list.appendChild(t);
	}
}

updateList(store.get("devices")[selectedDevice].profiles);
