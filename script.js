const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");
const spawn = require("child_process").spawn;
const RPC = require("discord-rpc");
const port = new SerialPort({ path: "/dev/ttyACM0", baudRate: 57600 });
const parser = port.pipe(new ReadlineParser({ delimiter: '\r\n' }));

(function() {
	let elements = document.getElementsByClassName("button-dropdown");
	for (let i = 0; i < elements.length; i++) {
		let dropdown = elements[i];
		dropdown.innerHTML = `
		<option value="clog"> Debug Log </option>
		<option value="dmute"> Discord Mute </option>
		<option value="ddeaf"> Discord Deafen </option>
		<option value="exec"> Launch Executable </option>
		`
		dropdown.addEventListener("change", function() {
			let option = document.getElementById(`option${this.id}`);
			option.disabled = true;
			option.value = "";
			switch (this.value) {
				case "clog": callbacks[this.id] = text; break;
				case "dmute": callbacks[this.id] = mute; break;
				case "ddeaf": callbacks[this.id] = deafen; break;
				case "exec": callbacks[this.id] = application; option.disabled = false; break;
			}
		});
}})();

var callbacks = {
	"1": text,
	"2": text,
	"3": text
}

const clientId = "1023592341484863638";
const scopes = [ "rpc", "rpc.voice.read", "rpc.voice.write" ];
const client = new RPC.Client({ transport: 'ipc' });

(async () => {
	let token = localStorage.getItem("discordToken");
	if (token) {
		client.login({ clientId, clientSecret, accessToken: localStorage.getItem("discordToken"), scopes, redirectUri: 'http://localhost:53134' });
	} else {
    	client.login({ clientId, clientSecret, scopes, redirectUri: 'http://localhost:53134' });
	}

    client.on('ready', () => {
        console.log('Authed for Discord user', client?.user?.username);
		localStorage.setItem("discordToken", client?.accessToken);
    });
})();

function text(dat) {
	console.log(`Button ${JSON.parse(dat).button} was pressed!`);
}

function mute(_) {
	client.getVoiceSettings().then((settings) => {
		client.setVoiceSettings({
			mute: !settings.mute
		});
	});
}
function deafen(_) {
	client.getVoiceSettings().then((settings) => {
		client.setVoiceSettings({
			deaf: !settings.deaf
		});
	});
}

function application(dat) {
	button = JSON.parse(dat).button;
	let child = spawn(document.getElementById(`option${button}`).value, [], {
		detached: true,
		stdio: [ 'ignore', 'ignore', 'ignore' ]
	});
	
	child.unref();
}

parser.on("data", function (dat) {
	callbacks[`${JSON.parse(dat).button}`](dat);
});