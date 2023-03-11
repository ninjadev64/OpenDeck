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
	if (port.vendorId === "2341" && port.productId === "0043") {
		serialPort.insertAdjacentHTML("beforeend", `<option value=${port.path}> ${port.path} </option>`);
	}
});
let webSocketPort = document.getElementById("websocket-port");
let propertyInspectorPort = document.getElementById("propertyinspector-port");
const options = {
	"serialPort": serialPort,
	"webSocketPort": webSocketPort,
	"propertyInspectorPort": propertyInspectorPort
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