if (require("electron-squirrel-startup")) process.exit(0);

import { BrowserWindow, Menu, Tray, app, ipcMain } from "electron";
if (!app.requestSingleInstanceLock()) app.exit();

import AutoLaunch from "auto-launch";
import path from "path";
import { ActionInstance, allActions, categories, setProfile, updateSlot } from "./shared";
import store from "./store";

let isQuitting = false;
let tray;

let mainWindow: BrowserWindow;

function createWindow(): void {
	mainWindow = new BrowserWindow({
		webPreferences: {
			nodeIntegration: true,
			contextIsolation: false,
		},
		autoHideMenuBar: true,
		icon: path.join(__dirname, "../src/assets/icon.png")
	});
  
	mainWindow.loadFile(path.join(__dirname, "../src/markup/index.html"));

	ipcMain.on("createInstance", (_event, action, device, type, position, index) => {
		let devices = store.get("devices");
		let instance = new ActionInstance(allActions[action], device, devices[device].selectedProfile, type, position, index);
		updateSlot(instance.context, instance);
		mainWindow.webContents.send("updateState", instance.context, instance);
	});
	
	ipcMain.on("slotUpdate", (_event, context, instance) => {
		updateSlot(context, instance);
		mainWindow.webContents.send("updateState", context, instance);
	});
	
	ipcMain.on("requestCategories", () => {
		mainWindow.webContents.send("categories", categories);
	});

	ipcMain.on("requestDevices", () => {
		mainWindow.webContents.send("devices", store.get("devices"));
	});

	ipcMain.on("requestProfiles", (_event, device) => {
		let d = store.get("devices")[device];
		mainWindow.webContents.send("profiles", d.profiles, d.selectedProfile);
	});

	ipcMain.on("createProfile", (_event, device, name, id) => {
		let devices = store.get("devices");
		devices[device].profiles[id] = {
			name,
			key: Array.from({ length: devices[device].keys }, () => [ null ]),
			slider: Array.from({ length: devices[device].sliders }, () => [ null ])
		};
		store.set("devices", devices);
		mainWindow.webContents.send("profiles", devices[device].profiles, devices[device].selectedProfile);
	});

	ipcMain.on("profileUpdate", (_event, device, id) => {
		setProfile(device, id);
		mainWindow.webContents.send("profiles", store.get("devices")[device].profiles, id);
	});

	mainWindow.on("close", (event: any) => {
		if (!isQuitting) {
			event.preventDefault();
			mainWindow.hide();
			event.returnValue = false;
		}
	});

	app.on("activate", () => {
		mainWindow.show();
		mainWindow.restore();
		mainWindow.focus();
	});

	app.on("second-instance", () => {
		mainWindow.show();
		mainWindow.restore();
		mainWindow.focus();
	});
}

app.whenReady().then(() => {
	require("./plugins");
	require("./devices");
	require("./propertyinspector");

	createWindow();

	tray = new Tray(path.join(__dirname, "../src/assets/icon.png"));
	tray.setContextMenu(Menu.buildFromTemplate([
		{
			label: "Open", click: () => {
				mainWindow.show();
			}
		},
		{
			label: "Quit", click: () => {
				isQuitting = true;
				app.exit();
			}
		}
	]));

	let autoLaunch = new AutoLaunch({
		name: "OpenDeck",
		isHidden: true
	});
	autoLaunch.isEnabled().then((isEnabled: boolean) => {
		if (store.get("autoLaunch") && !isEnabled) autoLaunch.enable();
		if (!store.get("autoLaunch") && isEnabled) autoLaunch.disable();
	});
});

app.on("before-quit", () => isQuitting = true);

export function getMainWindow(): BrowserWindow {
	return mainWindow;
}