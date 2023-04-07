const { BrowserWindow, ipcMain } = require("electron");
const { pluginManager } = require("./plugins");

const store = require("./store");
const { eventHandler } = require("./event");
const { getInstanceByContext } = require("./shared");
const WebSocketServer = require("ws").Server;

class PropertyInspector {
	constructor(instance) {
		this.action = instance.action;
		this.context = instance.context;
		this.path = this.action.propertyInspector;
		this.window = new BrowserWindow({
			autoHideMenuBar: true,
			width: 400,
			height: 250,
			show: false
		});
		this.window.loadFile(this.path);

		const info = pluginManager.plugins[this.action.plugin].info;
		const actionInfo = {
			action: this.action.uuid,
			context: this.context,
			device: 0,
			payload: {
				settings: {},
				coordinates: {
					row: Math.floor((this.context - 1) / 3),
					column: (this.context - 1) % 3
				},
				isInMultiAction: false
			}
		}

		this.window.once("ready-to-show", () => {
			this.window.title = this.action.name;
			this.window.webContents.executeJavaScript(`
				connectElgatoStreamDeckSocket(
					${store.get("propertyInspectorPort")},
					"${this.context}",
					"registerPropertyInspector",
					\`${JSON.stringify(info)}\`,
					\`${JSON.stringify(actionInfo)}\`
				);
			`);
		});
		this.window.on("close", (event) => {
			event.preventDefault();
			this.window.hide();
			eventHandler.propertyInspectorDidDisappear(instance);
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
			eventHandler.propertyInspectorDidAppear(getInstanceByContext(context));
		});
	}

	register(instance) {
		this.all[instance.context] = new PropertyInspector(instance);
	}

	unregister(instance) {
		let propertyInspector = this.all[instance.context];
		if (propertyInspector.socket != undefined) {
			propertyInspector.socket.close();
		}
		propertyInspector.window.destroy();
	}
}

const propertyInspectorManager = new PropertyInspectorManager();
module.exports = { propertyInspectorManager };