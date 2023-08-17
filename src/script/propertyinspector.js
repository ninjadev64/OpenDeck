const { BrowserWindow, ipcMain } = require("electron");
const { pluginManager } = require("./plugins");

const store = require("./store");
const { getInstanceByContext, getCoordinatesByContext, error } = require("./shared");
const WebSocketServer = require("ws").Server;

class PropertyInspector {
	constructor(instance) {
		this.action = instance.action;
		this.context = instance.context;
		this.path = this.action.propertyInspector;
		this.socket = null;
		this.queue = [];
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
				coordinates: getCoordinatesByContext(this.context),
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
			const { eventHandler } = require("./event");
			eventHandler.propertyInspectorDidDisappear(instance);
		});
	}

	send(data) {
		if (this.socket) {
			this.socket.send(data);
		} else {
			this.queue.push(data);
		}
	}

	setSocket(socket) {
		this.socket = socket;
		this.queue.forEach((item) => {
			this.socket.send(item);
			this.queue.shift();
		});
	}
}

class PropertyInspectorManager {
	constructor() {
		this.all = {};
		this.server = new WebSocketServer({ port: store.get("propertyInspectorPort") });
		this.server.on("error", () => {
			error("An error occurred. Is an instance of OpenDeck already running? Make sure your configured ports are free.", true);
			this.server.close();
		});
		this.server.on("connection", (ws) => {
			const { eventHandler } = require("./event");
			ws.on("message", (data) => {
				data = JSON.parse(data);
				if (data.event == "registerPropertyInspector") {
					this.all[data.uuid].setSocket(ws);
				} else {
					let f = eventHandler[data.event];
					if (f) f.bind(eventHandler)(data, true);
				}
			});
		});
		ipcMain.on("openPropertyInspector", (_event, context) => {
			this.all[context].window.show();
			const { eventHandler } = require("./event");
			eventHandler.propertyInspectorDidAppear(getInstanceByContext(context));
		});
	}

	register(instance) {
		this.all[instance.context] = new PropertyInspector(instance);
	}

	unregister(instance) {
		let propertyInspector = this.all[instance.context];
		if (propertyInspector.socket) {
			propertyInspector.socket.close();
		}
		propertyInspector.window.destroy();
	}

	async sendEvent(context, data) {
		this.all[context].send(JSON.stringify(data));
	}
}

const propertyInspectorManager = new PropertyInspectorManager();
module.exports = { propertyInspectorManager };