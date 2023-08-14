const { ipcRenderer } = require("electron");
const store = require("../script/store");

let list = document.getElementById("profiles");

document.getElementById("create").addEventListener("click", () => {
	let id = Date.now().toString(36) + Math.random().toString(36).substring(2);
	let name = document.getElementById("name").value;
	updateList({ ...(store.get("profiles")), [id]: { name } });
	ipcRenderer.send("createProfile", name, id);
	document.getElementById("name").value = "";
});

function updateList(profiles) {
	list.textContent = "";
	for (const [id, profile] of Object.entries(profiles)) {
		let t = document.createElement("li");
		t.innerText = profile.name;
		
		let i = document.createElement("img");
		i.src = "../assets/cross.png";
		i.className = "deleteProfile";
		i.addEventListener("click", () => {
			if (Object.keys(store.get("profiles")).length < 2) return;
			store.delete("profiles." + id);
			if (store.get("selectedProfile") == id) {
				ipcRenderer.send("profileUpdate", Object.keys(store.get("profiles"))[0]);
			} else {
				ipcRenderer.send("profileUpdate", store.get("selectedProfile"));
			}
			t.remove();
		});
		t.appendChild(i);
		
		list.appendChild(t);
	}
}

updateList(store.get("profiles"));