const os = require("os");
const fs = require("fs");
const path = require("path");
const store = require("./store");
const WebSocketServer = require("ws").Server;

const { allActions, categories, Action } = require("./shared");

const { app, BrowserWindow } = require("electron");
const { spawn } = require("child_process");
const { version } = require("../../package.json");

class StreamDeckPlugin {
	constructor(root, uuid) {
		let manifest = JSON.parse(fs.readFileSync(path.join(root, uuid, "manifest.json"), "utf8"));

		this.uuid = uuid;
		this.name = manifest.Name;
		this.description = manifest.Description;
		this.author = manifest.Author;
		this.version = manifest.Version;
		this.website = manifest.URL;
		this.iconPath = manifest.Icon;
		this.category = manifest.Category || "Custom";
		this.actions = [];
		this.socket = null;
		this.queue = [];
		this.propertyInspector = manifest.PropertyInspectorPath ? path.join(root, uuid, manifest.PropertyInspectorPath) : path.join(__dirname, "../markup/empty.html");
		
		if (categories[this.category] == undefined) categories[this.category] = [];
		manifest.Actions.forEach((action) => {
			let iconPath = path.join(root, uuid, action.Icon);
			let a = new Action(
				action.Name, action.UUID, this.uuid, action.Tooltip,
				fs.existsSync(iconPath + "@2x.png") ? iconPath + "@2x.png" : iconPath + ".png",
				action.PropertyInspectorPath ? path.join(root, uuid, action.PropertyInspectorPath) : this.propertyInspector,
				action.Controllers || [ "Keypad" ]
			);
			this.actions.push(a);
			allActions[a.uuid] = a;
			categories[this.category].push(a);
		});

		this.info = {
			"application": {
				"font": "Rubik",
				"language": "en",
				"platform": (
					os.platform() == "win32" ? "windows" :
					(os.platform() == "darwin" ? "mac" :
					(os.platform() == "linux" ? "linux" : "unknown"))
				),
				"platformVersion": os.version(),
				"version": version
			},
			"plugin": {
				"uuid": this.uuid,
				"version": this.version
			},
			"devicePixelRatio": 0,
			"colors": {
				"buttonPressedBackgroundColor": "#000000", 
				"buttonPressedBorderColor": "#000000", 
				"buttonPressedTextColor": "#000000", 
				"disabledColor": "#000000", 
				"highlightColor": "#000000", 
				"mouseDownColor": "#000000"
			},
			"devices": [
				{
					"id": "OceanDeck",
					"name": "OceanDeck",
					"size": {
						"columns": 3,
						"rows": 3
					},
					"type": 7
				}
			]
		}

		let codePath;
		switch (os.platform()) {
			case "win32":
				manifest.CodePathWin && (codePath = manifest.CodePathWin); break;
			case "darwin":
				manifest.CodePathMac && (codePath = manifest.CodePathMac); break;
			case "linux":
				manifest.CodePathLin && (codePath = manifest.CodePathLin); break;
		}
		if (codePath == undefined) codePath = manifest.CodePath;
		
		if (codePath.endsWith(".html")) {
			this.window = new BrowserWindow({
				autoHideMenuBar: true,
				icon: path.join(root, uuid, this.iconPath + ".png"),
				width: 600,
				height: 400,
				show: false
			});
			this.window.loadFile(path.join(root, uuid, codePath));
			this.window.once("ready-to-show", () => {
				this.window.title = this.name;
				this.window.webContents.executeJavaScript(`
					connectElgatoStreamDeckSocket(
						${store.get("webSocketPort")},
						"${this.uuid}",
						"register",
						\`${JSON.stringify(this.info)}\`
					);
				`);
			});
		} else {
			this.process = spawn(path.join(root, uuid, codePath), [
				"-port", store.get("webSocketPort"),
				"-pluginUUID", this.uuid,
				"-registerEvent", "register",
				"-info", JSON.stringify(this.info)
			]);
		}
	}

	send(data) {
		if (this.socket != null) {
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

class StreamDeckPluginManager {
	constructor() {
		this.pluginsDir = path.join(app.getPath("userData"), "Plugins");

		store.set("userDataPath", app.getPath("userData"));
		store.set("pluginsDir", this.pluginsDir);
		if (!fs.existsSync(this.pluginsDir)) fs.mkdirSync(this.pluginsDir);

		this.pluginIds = fs.readdirSync(this.pluginsDir, { withFileTypes: true })
			.filter((item) => item.isDirectory())
			.map((item) => item.name);
		this.plugins = {};
		
		this.server = new WebSocketServer({ port: store.get("webSocketPort") });
		this.server.on("connection", (ws) => {
			ws.on("message", (data) => {
				data = JSON.parse(data);
				if (data.event == "register") {
					this.plugins[data.uuid].setSocket(ws);
				}
			})
		});

		this.pluginIds.forEach((uuid) => {
			let pl = new StreamDeckPlugin(this.pluginsDir, uuid);
			this.plugins[uuid] = pl;
		});
	}

	async sendEvent(plugin, data) {
		this.plugins[plugin].send(JSON.stringify(data));
	}

	async sendGlobalEvent(data) {
		data = JSON.stringify(data);
		Object.values(this.plugins).forEach((plugin) => {
			plugin.send(data);
		});
	}
}

const pluginManager = new StreamDeckPluginManager();
module.exports = { pluginManager };