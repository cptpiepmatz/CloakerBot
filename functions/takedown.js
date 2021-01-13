"use strict";

const fs = require("fs");
const sleep = require("sleep-promise");
const path = require("path");

const getRandom = require("./getRandom");

const takedownFiles = fs.readdirSync(path.join(__dirname, "../sounds/takedowns/"));
const failFiles = fs.readdirSync(path.join(__dirname, "../sounds/fails/"));

/**
 * Kicks a user from a voice channel.
 * @param {GuildMember} member
 * @param {VoiceConnection} [connection]
 * @returns {Promise<void>}
 */
async function takedown(member, connection) {
  let takedownFile = path.join(__dirname, "../sounds/takedowns/", getRandom(takedownFiles));
  let failFile = path.join(__dirname, "../sounds/fails/", getRandom(failFiles));

  if (!connection) connection = await member.voice.channel.join();

  const wululuDispatcher = await connection.play(path.join(__dirname, "../sounds/WULULULULU.wav"), {volume: 0.3});
  await sleep(1000);
  await sleep(Math.floor(Math.random() * 3000));
  wululuDispatcher.pause(true);
  
  try {
    await member.voice.kick();
    console.log(`Took down ${member.user.username} on ${member.guild?.name}`);

    const takedownDispatcher = await connection.play(takedownFile, {volume: 0.5});
    takedownDispatcher.on("finish", () => {
      connection.disconnect();
    });
  }
  catch (e) {
    console.error(e);

    const failDispatcher = await connection.play(failFile, {volume: 0.5});
    failDispatcher.on("finish", async () => {
      connection.disconnect();
    });
  }
}

module.exports = takedown;