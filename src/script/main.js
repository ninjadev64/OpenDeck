const { app, ipcMain, BrowserWindow, Tray, Menu } = require("electron");
const path = require("path");

const { allActions, categories, updateKey, updateSlider, ActionInstance } = require("./shared");
const store = require("./store");

const AutoLaunch = require('auto-launch');

let isQuitting = false;
let tray;

let mainWindow;

const createWindow = () => {
	mainWindow = new BrowserWindow({
		webPreferences: {
			nodeIntegration: true,
			contextIsolation: false,
		},
		autoHideMenuBar: true,
		icon: path.join(__dirname, "../assets/icon.png")
	});
  
	mainWindow.loadFile(path.join(__dirname, "../markup/index.html"));

	ipcMain.on("createInstance", (_event, action, context, type) => {
		let instance = new ActionInstance(allActions[action], context, type);
		switch (type) {
			case "Keypad": updateKey(context, instance); break;
			case "Encoder": updateSlider(context, instance); break;
		}
		mainWindow.webContents.send("instanceCreated", instance);
	});
	
	ipcMain.on("keyUpdate", (_event, key, instance) => {
		updateKey(key, instance);
	});
	ipcMain.on("sliderUpdate", (_event, slider, instance) => {
		updateSlider(slider, instance);
	});
	
	ipcMain.on("requestCategories", () => {
		mainWindow.webContents.send("categories", categories);
	});

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
			}
		}
	]));

	mainWindow.on("close", (event) => {
		if (!isQuitting) {
			event.preventDefault();
			mainWindow.hide();
			event.returnValue = false;
		}
	});
}

app.whenReady().then(() => {
	createWindow();

	require("./plugins");
	require("./serial");
	require("./propertyinspector");

	let autoLaunch = new AutoLaunch({
		name: "OceanDesktop",
		isHidden: true
	});
	autoLaunch.isEnabled().then((isEnabled) => {
		if (store.get("autoLaunch") && !isEnabled) autoLaunch.enable();
		if (!store.get("autoLaunch") && isEnabled) autoLaunch.disable();
	});

	app.on("activate", () => {
		mainWindow.show();
	});
});

app.on("before-quit", () => {
	isQuitting = true;
});

function getMainWindow() {
	return mainWindow;
}

module.exports = { getMainWindow };