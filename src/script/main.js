const { app, ipcMain, BrowserWindow, Tray, Menu } = require("electron");
const { exit } = require("process");
const path = require("path");

const { allActions, categories, setProfile, updateSlot, ActionInstance } = require("./shared");
const store = require("./store");

const AutoLaunch = require('auto-launch');

let isQuitting = false;
let tray;

let mainWindow;

function createWindow() {
	mainWindow = new BrowserWindow({
		webPreferences: {
			nodeIntegration: true,
			contextIsolation: false,
		},
		autoHideMenuBar: true,
		icon: path.join(__dirname, "../assets/icon.png")
	});
  
	mainWindow.loadFile(path.join(__dirname, "../markup/index.html"));

	ipcMain.on("createInstance", (_event, action, type, position, index) => {
		let instance = new ActionInstance(allActions[action], store.get("selectedProfile"), type, position, index);
		updateSlot(instance.context, instance);
		mainWindow.webContents.send("instanceCreated", instance);
	});
	
	ipcMain.on("slotUpdate", (_event, context, instance) => {
		updateSlot(context, instance);
	});
	
	ipcMain.on("requestCategories", () => {
		mainWindow.webContents.send("categories", categories);
	});

	ipcMain.on("requestProfiles", () => {
		mainWindow.webContents.send("profiles", store.get("profiles"), store.get("selectedProfile"));
	});

	ipcMain.on("createProfile", (_event, name, id) => {
		store.set("profiles." + id, {
			name,
			key: [ [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ] ],
			slider: [ [ null ], [ null ] ]
		});
		mainWindow.webContents.send("profiles", store.get("profiles"), store.get("selectedProfile"));
		return 0;
	});

	ipcMain.on("profileUpdate", (_event, id) => {
		setProfile(id);
		mainWindow.webContents.send("profiles", store.get("profiles"), id);
	});

	mainWindow.on("close", (event) => {
		if (!isQuitting) {
			event.preventDefault();
			mainWindow.hide();
			event.returnValue = false;
		}
	});

	app.on("activate", () => {
		mainWindow.show();
	});
}

app.whenReady().then(() => {
	createWindow();

	tray = new Tray(path.join(__dirname, "../assets/icon.png"));
	tray.setContextMenu(Menu.buildFromTemplate([
		{
			label: "Open", click: () => {
				mainWindow.show();
			}
		},
		{
			label: "Quit", click: () => {
				isQuitting = true;
				app.quit();
				exit(0);
			}
		}
	]));

	require("./plugins");
	require("./serial");
	require("./propertyinspector");

	let autoLaunch = new AutoLaunch({
		name: "OpenDeck",
		isHidden: true
	});
	autoLaunch.isEnabled().then((isEnabled) => {
		if (store.get("autoLaunch") && !isEnabled) autoLaunch.enable();
		if (!store.get("autoLaunch") && isEnabled) autoLaunch.disable();
	});
});

app.on("before-quit", () => {
	isQuitting = true;
});

function getMainWindow() {
	return mainWindow;
}

module.exports = { getMainWindow };