"use strict";

const path = require("path");
const Canvas = require("canvas");

const templatePath = path.join(__dirname, "../assets/graphics/user_rank.png");
let templateImage;

/**
 * Generates the graphic for the user rank badge.
 *
 * <p>All the pairs represent first the guild value and second the total value.
 *
 * @param {GuildMember} member The member to generate the graphic for
 * @param {[number, number]} victimRank Pair of victim rank
 * @param {[number, number]} punisherRank Pair of punisher rank
 * @param {[number, number]} timesSmacked Pair of times smacked
 * @param {[number, number]} timesCalled Pair of times called
 */
async function generateUserRankGraphic(
  member,
  victimRank,
  punisherRank,
  timesSmacked,
  timesCalled
) {
  if (!templateImage) templateImage = await Canvas.loadImage(templatePath);

  let canvas = Canvas.createCanvas(600, 150);
  let ctx = canvas.getContext("2d");
  ctx.drawImage(templateImage, 600, 150);
  return canvas.toDataURL();
}

module.exports = generateUserRankGraphic;
