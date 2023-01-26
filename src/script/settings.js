const Store = require("electron-store");
const store = new Store();

const { platform } = require("os");
const { spawn } = require("child_process");

document.getElementById("open-plugins").addEventListener("click", () => {
	var explorer;
	switch (platform()) {
		case "win32": explorer = "explorer"; break;
		case "linux": explorer = "xdg-open"; break;
		case "darwin": explorer = "open"; break;
	}
	spawn(explorer, [store.get("pluginsDir")], { detached: true }).unref();
});

let serialSelect = document.getElementById("serial-port");
store.get("allPorts").forEach((port) => {
	if (port.vendorId === "2341" && port.productId === "0043") {
		serialSelect.insertAdjacentHTML("beforeend", `<option value=${port.path}> ${port.path} </option>`);
	}
});
const options = {
	"serialPort": serialSelect
}
for (const [key, value] of Object.entries(options)) {
	value.value = store.get(key);
}
function applyChanges() {
	for (const [key, value] of Object.entries(options)) {
		store.set(key, value.value);
	}
	alert("Changes have been applied. You may need to restart OceanDesktop for them to take effect.");
}
document.getElementById("apply-changes").addEventListener("click", applyChanges);