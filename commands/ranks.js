"use strict";

const Discord = require("discord.js");

const fetchMemberRanks = require("../functions/fetchMemberRanks");
const generateUserRankGraphic = require("../functions/generateUserRankGraphic");
const getRandom = require("../functions/getRandom");

const replies = require("../server/replies");

/**
 * Generates a rank card and posts it in the channel.
 *
 * @param {Message} message
 * @returns {Promise<void>}
 */
async function ranks(message) {
  if (!message.member.hasPermission("ATTACH_FILES")) {
    message.channel.send(getRandom(replies.noPerms)).catch(console.error);
    return;
  }

  let args = message.content.trim().toLowerCase().split(/\s+/);
  let whoToRank;
  if (args.length === 2) {
    // if no one is defined to rank for, use the author
    whoToRank = message.author.id;
  }
  else {
    // try to get who to rank
    whoToRank = message.mentions.users.firstKey() || args[2];
  }

  let fetchedRankedMember = await fetchMemberRanks(whoToRank, message.guild);
  if (!fetchedRankedMember) {
    message.channel.send(getRandom(replies.noMember)).catch(console.error);
    return;
  }

  console.log("Generating rank image for " + fetchedRankedMember.UserName);
  console.log(fetchedRankedMember);

  let imageBuffer = await generateUserRankGraphic(
    await message.guild.members.fetch(whoToRank),
    [
      fetchedRankedMember.victimRankHere,
      fetchedRankedMember.victimRankTotal
    ],
    [
      fetchedRankedMember.punisherRankHere,
      fetchedRankedMember.punisherRankTotal
    ],
    [
      fetchedRankedMember.timesSmackedHere,
      fetchedRankedMember.timesSmackedTotal
    ],
    [
      fetchedRankedMember.timesCalledHere,
      fetchedRankedMember.timesCalledTotal
    ]
  );

  let attachment = new Discord.MessageAttachment(
    imageBuffer,
    `Rank ${fetchedRankedMember.UserName} ${new Date()}.png`
  );
  message.channel.send({files: [attachment]}).catch(console.error);
}

module.exports = ranks;
