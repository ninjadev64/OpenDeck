const store = require("./store");
const { BrowserWindow } = require("electron");
const { keys } = require("./shared");
const { pluginManager } = require("./plugins");

class PropertyInspector {
    constructor(action, path, key) {
        this.action = action;
        this.window = new BrowserWindow({
            autoHideMenuBar: true,
            width: 300,
            height: 200,
            // show: false
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
            this.window.title = this.name;
            this.window.webContents.executeJavaScript(`
            connectElgatoStreamDeckSocket(${store.get("propertyInspectorPort")}, ${key}, "registerPropertyInspector", \`${JSON.stringify(info)}\`, \`${JSON.stringify(actionInfo)}\`);
            `);
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
					this.all[data.uuid].socket = ws;
				}
			})
		});
    }

    register(key) {
        if (this.all[key] != undefined) {
            this.all[key].socket.close();
        }
        this.all[key] = new PropertyInspector(keys[key], action.propertyInspector, key);
    }
}

const propertyInspectorManager = new PropertyInspectorManager();
module.exports = { propertyInspectorManager };