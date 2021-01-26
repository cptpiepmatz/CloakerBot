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
  console.log(process._getActiveHandles().length, " begin takedown");

  let takedownFile = path.join(__dirname, "../sounds/takedowns/", getRandom(takedownFiles));
  let failFile = path.join(__dirname, "../sounds/fails/", getRandom(failFiles));

  if (!connection) connection = await member.voice.channel.join();

  console.log(process._getActiveHandles().length, " before wululu");
  // min   60000
  // max  230000
  // diff 170000
  let wululuTime = 60000 + Math.floor(Math.random() * 170000);
  let wululuStream = fs.createReadStream(
    path.join(__dirname, "../sounds/WULULULULU_reduced_volume.wav"),
    {end: wululuTime}
  );
  const wululuDispatcher = await connection.play(wululuStream);
  console.log("opened wululuDispatcher");
  console.log(process._getActiveHandles().length, " wululu opened");
  wululuDispatcher.on("finish", async () => {
    endIfPossible(wululuDispatcher, "wululuDispatcher");
    console.log(process._getActiveHandles().length, " wululu destroyed");

    let takedownDispatcher;
    try {
      await member.voice.kick();
      console.log(`Took down ${member.user.username} on ${member.guild?.name}`);

      takedownDispatcher = await connection.play(takedownFile, {volume: 0.5, bitrate: "auto"});
      console.log(process._getActiveHandles().length, " takedown sound online");
      console.log("opened takedownDispatcher");
      takedownDispatcher.on("finish", () => {
        connection.disconnect();
        endIfPossible(takedownDispatcher, "takedownDispatcher");
        console.log(process._getActiveHandles().length, "takedown sound end");
      });
      takedownDispatcher.on("error", () => {
        connection.disconnect();
        endIfPossible(takedownDispatcher, "takedownDispatcher");
      });
    }
    catch (e) {
      console.error(e);

      const failDispatcher = await connection.play(failFile, {volume: 0.5});
      console.log("opened failDispatcher");
      failDispatcher.on("finish", async () => {
        endIfPossible(failDispatcher, "failDispatcher");
        endIfPossible(takedownDispatcher, "takedownDispatcher");
        connection.disconnect();
      });
      failDispatcher.on("error", async () => {
        connection.disconnect();
        endIfPossible(failDispatcher, "failDispatcher");
        endIfPossible(takedownDispatcher, "takedownDispatcher");
      })
    }
  });
}

// Try to destroy a dispatcher, if it has the destroy method
function endIfPossible(dispatcher, name = "") {
  if (typeof dispatcher?.destroy === "function") {
    dispatcher.end();
    if (dispatcher.destroyed) {
      console.log("closed " + name);
    }
  }
}

module.exports = takedown;