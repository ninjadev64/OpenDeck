const { BrowserWindow, ipcMain } = require("electron");
const { keys } = require("./shared");
const { pluginManager } = require("./plugins");

const store = require("./store");
const { eventHandler } = require("./event");
const WebSocketServer = require("ws").Server;

class PropertyInspector {
	constructor(action, path, key) {
		this.action = action;
		this.window = new BrowserWindow({
			autoHideMenuBar: true,
			width: 400,
			height: 250,
			show: false
		});
		this.window.loadFile(path);

		const info = pluginManager.plugins[action.plugin].info;
		const actionInfo = {
			action: action.uuid,
			context: key,
			device: 0,
			payload: {
				settings: {},
				coordinates: {
					row: Math.floor(key / 3) + 1,
					column: key % 3
				},
				isInMultiAction: false
			}
		}

		this.window.once("ready-to-show", () => {
			this.window.title = action.name;
			this.window.webContents.executeJavaScript(`
				connectElgatoStreamDeckSocket(
					${store.get("propertyInspectorPort")},
					${key},
					"registerPropertyInspector",
					\`${JSON.stringify(info)}\`,
					\`${JSON.stringify(actionInfo)}\`
				);
			`);
		});
		this.window.on("close", (event) => {
			event.preventDefault();
			this.window.hide();
			eventHandler.propertyInspectorDidDisappear(key);
		});
	}
}

class PropertyInspectorManager {
	constructor() {
		this.all = {};
		this.server = new WebSocketServer({ port: store.get("propertyInspectorPort") });
		this.server.on("connection", (ws) => {
			ws.on("message", (data) => {
				data = JSON.parse(data);
				if (data.event == "registerPropertyInspector") {
					this.all[parseInt(data.uuid)].socket = ws;
				}
			})
		});
		ipcMain.on("openPropertyInspector", (_event, key) => {
			this.all[key].window.show();
			eventHandler.propertyInspectorDidAppear(key);
		});
	}

	register(key) {
		this.all[key] = new PropertyInspector(keys[key], keys[key].propertyInspector, key);
	}

	unregister(key) {
		if (this.all[key].socket != undefined) {
			this.all[key].socket.close();
		}
		this.all[key].window.destroy();
	}
}

const propertyInspectorManager = new PropertyInspectorManager();
module.exports = { propertyInspectorManager };