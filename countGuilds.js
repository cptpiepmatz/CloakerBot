const Discord = require("discord.js");

const token = require("./server/token");

const client = new Discord.Client();

client.login(token).then(() => {
  console.log("The bot is currently active on "
    + client.guilds.cache.keyArray().length
    + " guilds");
  client.destroy();
});