const wtf = require("wtfnode");
const util = require("util");
const Socket = require("net").Socket;

const Discord = require("discord.js");
const { setIntervalAsync } = require("set-interval-async/fixed");

const randomTakedown = require("./functions/randomTakedown");
const openDatabase = require("./functions/openDatabase");
const purgeDatabase = require("./functions/purgeDatabase");

const commandTakedown = require("./commands/takedown");
const commandHelp = require("./commands/help");
const commandRandom = require("./commands/random");

const token = require("./server/token");

const client = new Discord.Client({
  messageCacheLifetime: 3 * 60,
  messageSweepInterval: 30
});

// startup
(async function() {
  await openDatabase();
  await client.login(token);
  console.log("Done.");
})();

client.on("message", async message => {
  if (message.content === "wtf-dump") {
    wtf.dump();
  }

  if (!message.guild) return;
  if (message.author.bot) return;
  if (!message.content.toLowerCase().startsWith("/cloaker")) return;

  let args = message.content.trim().toLowerCase().split(/\s+/);

  try {
    switch (args[1]) {
      case "help":
        await commandHelp(message);
        break;
      case "random":
        await commandRandom(message);
        break;
      default:
        console.log("-------");
        console.log(process._getActiveHandles().length, " before command");
        await commandTakedown(message);
    }
  }
  catch (e) {
    console.error(e);
  }
});

// When the bot is kicked from a guild, delete it's data stored in the db
client.on("guildDelete", async guild => {
  try {
    await purgeDatabase(guild);
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
