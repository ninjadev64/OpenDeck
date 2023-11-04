import { BrowserWindow, ipcMain } from "electron";
import { Server as WebSocketServer } from "ws";
import { pluginManager } from "./plugins";
import { Action, ActionInstance, error, getCoordinatesByContext, getInstanceByContext } from "./shared";
import store from "./store";

export class PropertyInspector {
	action: Action;
	context: string;
	path: string;
	socket: any;
	queue: any[];
	window: BrowserWindow;

	constructor(instance: ActionInstance) {
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

	send(data: string): void {
		if (this.socket) {
			this.socket.send(data);
		} else {
			this.queue.push(data);
		}
	}

	setSocket(socket: any): void {
		this.socket = socket;
		this.queue.forEach((item) => {
			this.socket.send(item);
			this.queue.shift();
		});
	}
}

class PropertyInspectorManager {
	all: { [ uuid: string ]: PropertyInspector };
	server: WebSocketServer;

	constructor() {
		this.all = {};
		this.server = new WebSocketServer({ port: store.get("propertyInspectorPort") });
		this.server.on("error", () => {
			error("An error occurred. Try removing any recently installed plugins, and make sure your configured ports are free.", true);
			this.server.close();
		});
		this.server.on("connection", (ws: any) => {
			const { eventHandler } = require("./event");
			ws.on("message", (message: string) => {
				let data = JSON.parse(message);
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

	register(instance: ActionInstance): void {
		this.all[instance.context] = new PropertyInspector(instance);
	}

	unregister(instance: ActionInstance): void {
		let propertyInspector = this.all[instance.context];
		if (propertyInspector.socket) {
			propertyInspector.socket.close();
		}
		propertyInspector.window.destroy();
	}

	async sendEvent(context: string, data: object): Promise<void> {
		this.all[context].send(JSON.stringify(data));
	}
}

export const propertyInspectorManager = new PropertyInspectorManager();
