const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");

if (!localStorage.getItem("serialPort")) localStorage.setItem("serialPort", "/dev/oceandeck");

const port = new SerialPort({ path: localStorage.getItem("serialPort"), baudRate: 57600 });
const parser = port.pipe(new ReadlineParser({ delimiter: '\r\n' }));

document.getElementById("open-settings").addEventListener("click", () => {
	window.open("settings.html");
});

const basicActions = new BasicActions();
const discordActions = new DiscordActions();

var actions = {
	"clog": { name: "Debug Log", handler: (d) => { basicActions.text(d); } },
	"dmute": { name: "Discord Mute", handler: (d) => { discordActions.mute(d); } },
	"ddeaf": { name: "Discord Deafen", handler: (d) => { discordActions.deafen(d); } },
	"exec": { name: "Launch Executable", handler: (d) => { basicActions.application(d); }, additionalInput: true },
	"keyc": { name: "Key Combination", handler: (d) => { basicActions.keyCombo(d); }, additionalInput: true }
};

var callbacks = {
	0: (d) => { return d; },
	1: (d) => { basicActions.text(d); },
	2: (d) => { basicActions.text(d); },
	3: (d) => { basicActions.text(d); }
};

function hexToRgb(hex) {
	// https://stackoverflow.com/a/5624139/14269655
	var shorthandRegex = /^#?([a-f\d])([a-f\d])([a-f\d])$/i;
	hex = hex.replace(shorthandRegex, function(m, r, g, b) {
		return r + r + g + g + b + b;
	});

	var result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
	return {
		red: parseInt(result[1], 16),
		green: parseInt(result[2], 16),
		blue: parseInt(result[3], 16)
	};
}

document.getElementById("led1").addEventListener("change", function () {
	port.write(JSON.stringify(hexToRgb(this.value)) + "\n");
});

(function() {
	let elements = document.getElementsByClassName("button-dropdown");
	for (let i = 0; i < elements.length; i++) {
		let dropdown = elements[i];
		let optionHTML = "";
		for (const [key, value] of Object.entries(actions)) {
			optionHTML+=`<option value="${key}"> ${value.name} </option>`;
		}
		dropdown.innerHTML = optionHTML;
		dropdown.addEventListener("change", function() {
			let option = document.getElementById(`option${this.id}`);
			option.disabled = true;
			option.value = "";
			callbacks[this.id] = (d) => {
				try {
					actions[this.value].handler(d);
				} catch (e) {
					alert(`Action ${actions[this.value].name} failed with error message ${e.message}`);
				}
			};
			if (actions[this.value].additionalInput) option.disabled = false;
		});
}})();

parser.on("data", function (dat) {
	dat = JSON.parse(dat);	
	callbacks[dat.button](dat);
});