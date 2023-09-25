const store = require("../../build/store").default;

const { platform } = require("os");
const { spawn } = require("child_process");
const dialog = require("dialog");

document.getElementById("open-plugins").addEventListener("click", () => {
	var explorer;
	switch (platform()) {
		case "win32": explorer = "explorer"; break;
		case "linux": explorer = "xdg-open"; break;
		case "darwin": explorer = "open"; break;
	}
	spawn(explorer, [store.get("pluginsDir")], { detached: true }).unref();
});

let autoLaunch = document.getElementById("auto-launch");
let useBluetoothProntoKey = document.getElementById("use-bluetooth");
let webSocketPort = document.getElementById("websocket-port");
let propertyInspectorPort = document.getElementById("propertyinspector-port");
const options = {
	"autoLaunch": autoLaunch,
	"useBluetoothProntoKey": useBluetoothProntoKey,
	"webSocketPort": webSocketPort,
	"propertyInspectorPort": propertyInspectorPort
}
for (const [key, value] of Object.entries(options)) {
	switch (value.type) {
		case "checkbox": value.checked = store.get(key);
		default: value.value = store.get(key);
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
	dialog.info("Changes have been applied. You may need to restart OpenDeck for them to take effect.", "Success");
}
document.getElementById("apply-changes").addEventListener("click", applyChanges);