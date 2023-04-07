const { BrowserWindow, ipcMain } = require("electron");
const { keys, sliders } = require("./shared");
const { pluginManager } = require("./plugins");

const store = require("./store");
const { eventHandler } = require("./event");
const WebSocketServer = require("ws").Server;

class PropertyInspector {
	constructor(action, path, context) {
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
			context: context,
			device: 0,
			payload: {
				settings: {},
				coordinates: {
					row: Math.floor((context - 1) / 3),
					column: (context - 1) % 3
				},
				isInMultiAction: false
			}
		}

		this.window.once("ready-to-show", () => {
			this.window.title = action.name;
			this.window.webContents.executeJavaScript(`
				connectElgatoStreamDeckSocket(
					${store.get("propertyInspectorPort")},
					"${context}",
					"registerPropertyInspector",
					\`${JSON.stringify(info)}\`,
					\`${JSON.stringify(actionInfo)}\`
				);
			`);
		});
		this.window.on("close", (event) => {
			event.preventDefault();
			this.window.hide();
			eventHandler.propertyInspectorDidDisappear(context, action);
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
					if (data.uuid.startsWith("s")) {
						this.all[data.uuid].socket = ws;
					} else {
						this.all[parseInt(data.uuid)].socket = ws;
					}
				}
			})
		});
		ipcMain.on("openPropertyInspector", (_event, context) => {
			this.all[context].window.show();
			eventHandler.propertyInspectorDidAppear(context, this.all[context].action);
		});
	}

	registerKey(key) {
		this.all[key] = new PropertyInspector(keys[key], keys[key].propertyInspector, key);
	}

	registerSlider(slider) {
		this.all[`s${slider}`] = new PropertyInspector(sliders[slider], sliders[slider].propertyInspector, `s${slider}`);
	}

	unregister(context) {
		if (this.all[context].socket != undefined) {
			this.all[context].socket.close();
		}
		this.all[context].window.destroy();
	}
}

const propertyInspectorManager = new PropertyInspectorManager();
module.exports = { propertyInspectorManager };