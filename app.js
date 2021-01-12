const Discord = require("discord.js");
const { setIntervalAsync } = require("set-interval-async/fixed");
const sleep = require("sleep-promise");
const fs = require("fs");
const path = require("path");

const commandTakedown = require("./functions/commandTakedown");
const randomTakedown = require("./functions/randomTakedown");
const openDatabase = require("./functions/openDatabase");

const client = new Discord.Client();

let token = fs.readFileSync(path.join(__dirname, ".token"), "utf8");
const args = process.argv.slice(2);
if (args.includes("test")) {
  token = fs.readFileSync(path.join(__dirname, "test.token"), "utf8");
}

// startup
(async function() {
  await openDatabase();
  await client.login(token);
  console.log("Done.");
})();

client.on("message", async message => {
  if (!message.guild) return;
  if (message.author.bot) return;
  if (!message.content.toLowerCase().startsWith("/cloaker")) return;

  try {
    await commandTakedown(message);
  }
  catch (e) {
    console.error(e);
  }
});

// Min Cooldown for about 4 hours
const loop = setIntervalAsync(
  async () => {
    await randomTakedown(client);
  },
  14400000
);
