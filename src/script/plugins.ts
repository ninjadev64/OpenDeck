import { ChildProcessWithoutNullStreams, execSync, spawn } from "child_process";
import { BrowserWindow, app } from "electron";
import fs from "fs";
import os from "os";
import path from "path";
import { Server as WebSocketServer } from "ws";
import { Action, ActionState, allActions, categories, error, getIcon } from "./shared";
import store from "./store";

const version = "1.0.0";

class StreamDeckPlugin {
	uuid: string;
	name: string;
	description: string;
	author: string;
	version: string;
	website: string;
	iconPath: string;
	category: string;
	actions: Action[];
	socket: any;
	queue: string[];
	propertyInspector: string;
	applicationsToMonitor: string[];
	info: {
		application: object;
		plugin: object;
		devicePixelRatio: number;
		colors: object;
		devices: object[];
	};
	window: BrowserWindow;
	process: ChildProcessWithoutNullStreams;

	constructor(root: string, uuid: string) {
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
		this.propertyInspector = manifest.PropertyInspectorPath ? path.join(root, uuid, manifest.PropertyInspectorPath) : path.join(__dirname, "../src/markup/empty.html");

		this.applicationsToMonitor = [];
		if (manifest.ApplicationsToMonitor) {
			switch (os.platform()) {
				case "win32": this.applicationsToMonitor = manifest.ApplicationsToMonitor.windows ?? []; break;
				case "darwin": this.applicationsToMonitor = manifest.ApplicationsToMonitor.mac ?? []; break;
				case "linux": this.applicationsToMonitor = manifest.ApplicationsToMonitor.linux ?? []; break;
			}
		}
		
		if (categories[this.category] == undefined) categories[this.category] = [];
		manifest.Actions.forEach((action: any) => {
			if (!action.Icon) action.Icon = action.States[0].Image;
			let iconPath = getIcon(path.join(root, uuid, action.Icon));
			let states: ActionState[] = [];
			action.States.forEach((state: any) => {
				if (!state.Image || state.Image == "actionDefaultImage") {
					state.Image = iconPath;
				} else {
					state.Image = getIcon(path.join(root, uuid, state.Image));
				}
				states.push(new ActionState(state, action.Name));
			});
			let a = new Action(
				action.Name, action.UUID, this.uuid, action.Tooltip,
				iconPath,
				action.PropertyInspectorPath ? path.join(root, uuid, action.PropertyInspectorPath) : this.propertyInspector,
				action.Controllers || [ "Keypad" ],
				states,
				action.VisibleInActionsList == false ? false : true
			);
			this.actions.push(a);
			allActions[a.uuid] = a;
			categories[this.category].push(a);
		});

		let devices = [];
		for (const [id, data] of Object.entries(store.get("devices"))) {
			let details = data as any;
			devices.push({
				id: id,
				name: details.name,
				size: {
					rows: details.rows,
					columns: details.columns
				},
				type: details.type
			});
		}
		const platform = os.platform();
		this.info = {
			"application": {
				"font": "Rubik",
				"language": "en",
				"platform": (
					platform == "win32" ? "windows" :
					(platform == "darwin" ? "mac" :
					(platform == "linux" ? "linux" : "unknown"))
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
			"devices": devices
		}

		let codePath = manifest.CodePath ?? "";
		switch (platform) {
			case "win32": manifest.CodePathWin && (codePath = manifest.CodePathWin); break;
			case "darwin": manifest.CodePathMac && (codePath = manifest.CodePathMac); break;
			case "linux": manifest.CodePathLin && (codePath = manifest.CodePathLin); break;
		}
		let supportsWindows = false;
		let enableWine = false;
		let supported = false;
		manifest.OS.forEach(({ Platform }: { Platform: string }) => {
			if (Platform == "windows") supportsWindows = true;
			if (
				(platform == "win32" && Platform == "windows") ||
				(platform == "darwin" && Platform == "mac") ||
				(platform == "linux" && Platform == "linux")
			) {
				supported = true;
				return false;
			}
		});
		if (!supported && codePath.endsWith(".html")) supported = true;
		wine: if (!supported && platform != "win32" && supportsWindows) {
			const winePath = execSync(`which wine`).toString().trim();
			if (!winePath || winePath.includes("not found")) break wine;
			if (!codePath) codePath = manifest.CodePathWin;
			if (!codePath.endsWith(".exe")) codePath += ".exe";
			enableWine = true;
			supported = true;
		}
		if (!supported) {
			error(`The plugin ${uuid} is not supported on the platform "${platform}"!`, false);
			return;
		}

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
			if (["darwin", "linux"].includes(platform)) execSync(`chmod +x "${path.join(root, uuid, codePath)}"`);
			if (enableWine) {
				this.process = spawn(
					"wine",
					[
						path.join(root, uuid, codePath),
						"-port", store.get("webSocketPort"),
						"-pluginUUID", this.uuid,
						"-registerEvent", "register",
						"-info", JSON.stringify(this.info)
					],
					{ cwd: path.join(root, uuid) }
				);
			} else {
				this.process = spawn(
					path.join(root, uuid, codePath),
					[
						"-port", store.get("webSocketPort"),
						"-pluginUUID", this.uuid,
						"-registerEvent", "register",
						"-info", JSON.stringify(this.info)
					],
					{ cwd: path.join(root, uuid) }
				);
			}
		}
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

class StreamDeckPluginManager {
	pluginsDir: string;
	pluginIds: string[];
	plugins: { [uuid: string]: StreamDeckPlugin };
	server: WebSocketServer;
	applicationMonitors: { [id: string]: StreamDeckPlugin[] };
	applicationCounts: { [id: string]: number };
	lastPollTime: number;
	bundleIDs: { [id: string]: string };

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
		this.server.on("error", () => {
			error("An error occurred. Try removing any recently installed plugins, and make sure your configured ports are free.", true);
			this.server.close();
		});

		this.server.on("connection", (ws: any) => {
			const { eventHandler } = require("./event");
			ws.on("message", (message: string) => {
				let data = JSON.parse(message);
				if (data.event == "register") {
					this.plugins[data.uuid].setSocket(ws);
				} else {
					let f = eventHandler[data.event];
					if (f) f.bind(eventHandler)(data, false);
				}
			});
		});

		this.pluginIds.forEach((uuid) => {
			let pl = new StreamDeckPlugin(this.pluginsDir, uuid);
			this.plugins[uuid] = pl;
		});

		this.applicationMonitors = {};
		this.applicationCounts = {};
		this.lastPollTime = Infinity;
		if (os.platform() == "darwin") this.bundleIDs = store.get("bundleIDs");
		Object.values(this.plugins).forEach((plugin) => {
			plugin.applicationsToMonitor.forEach((application) => {
				if (!this.applicationMonitors[application]) this.applicationMonitors[application] = [];
				this.applicationMonitors[application].push(plugin);
				this.applicationCounts[application] = 0;
			});
		});
		import("ps-list").then((pslist) => {
			setInterval(async () => {
				let now = Date.now();
				if (now > (this.lastPollTime + 2500)) {
					this.sendGlobalEvent({ event: "systemDidWakeUp" });
				}
				this.lastPollTime = now;
				
				let counts: { [id: string]: number } = {};
				let processes = await pslist.default();
				processes.forEach((process) => {
					let p = process.name;
					if (os.platform() == "darwin") {
						let s = process.cmd.split("/Contents/MacOS");
						if (s.length < 2) return;
						if (!this.bundleIDs[s[0]]) {
							try { this.bundleIDs[s[0]] = execSync(`defaults read "${s[0]}/Contents/Info.plist" CFBundleIdentifier`).toString().trim(); }
							catch (err) { this.bundleIDs[s[0]] = "No bundle ID"; }
							store.set("bundleIDs", this.bundleIDs);
						}
						p = this.bundleIDs[s[0]];
					}
					if (!this.applicationMonitors[p]) return;
					if (!counts[p]) counts[p] = 0;
					counts[p] += 1;
				});

				const { eventHandler } = require("./event");
				for (const [key, value] of Object.entries(this.applicationCounts)) {
					if (!counts[key]) counts[key] = 0;
					if (counts[key] == value) continue;
					this.applicationMonitors[key].forEach((plugin) => {
						for (let i = 0; i < Math.abs(counts[key] - value); i++) {
							if (value < counts[key]) eventHandler.applicationDidLaunch(key, plugin.uuid);
							else eventHandler.applicationDidTerminate(key, plugin.uuid);
						}
					});
					this.applicationCounts[key] = counts[key];
				}
			}, 1000);
		});
	}

	async sendEvent(plugin: string, data: object): Promise<void> {
		this.plugins[plugin].send(JSON.stringify(data));
	}

	async sendGlobalEvent(data: object): Promise<void> {
		let stringified = JSON.stringify(data);
		Object.values(this.plugins).forEach((plugin) => {
			plugin.send(stringified);
		});
	}
}

export const pluginManager = new StreamDeckPluginManager();