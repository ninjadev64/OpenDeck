const store = require("../script/store");

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

let serialPort = document.getElementById("serial-port");
store.get("allPorts").forEach((port) => {
	if (port.vendorId === "10c4" && port.productId === "ea60") {
		serialPort.insertAdjacentHTML("beforeend", `<option value=${port.path}> ${port.path} </option>`);
	}
});
let autoLaunch = document.getElementById("autolaunch");
let webSocketPort = document.getElementById("websocket-port");
let propertyInspectorPort = document.getElementById("propertyinspector-port");
const options = {
	"serialPort": serialPort,
	"autoLaunch": autoLaunch,
	"webSocketPort": webSocketPort,
	"propertyInspectorPort": propertyInspectorPort
}
for (const [key, value] of Object.entries(options)) {
	switch (value.type) {
		case "checkbox": value.checked = store.get(key);
		default: value.value = store.get(key).toString();
	}
}
function applyChanges() {
	for (const [key, value] of Object.entries(options)) {
		switch (value.type) {
			case "number": store.set(key, parseInt(value.value)); break;
			case "checkbox": store.set(key, value.checked); break;
			default: store.set(key, value.value); break;
		}
	}
	alert("Changes have been applied. You may need to restart OceanDesktop for them to take effect.");
}
document.getElementById("apply-changes").addEventListener("click", applyChanges);