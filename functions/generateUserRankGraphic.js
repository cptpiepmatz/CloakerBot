"use strict";

const path = require("path");
const Canvas = require("canvas");

const templatePath = path.join(__dirname, "../assets/graphics/user_rank.png");
let templateImage;

// Register the free to use font Roboto
const fontFamily = '"Roboto", "Cambria Math", "Segoe UI", sans-serif';

const robotoPath = path.join(__dirname, "../assets/fonts/Roboto/");
Canvas.registerFont(robotoPath + "Roboto-Regular.ttf", {
  family: "Roboto",
  weight: "400"
});
Canvas.registerFont(robotoPath + "Roboto-Bold.ttf", {
  family: "Roboto",
  weight: "700"
});

const segoePath = path.join(__dirname, "../assets/fonts/Segoe/");
Canvas.registerFont(segoePath + "segoeui.ttf", {
  family: "Segoe UI",
  weight: "400"
});
Canvas.registerFont(segoePath + "segoeuib.ttf", {
  family: "Segoe UI",
  weight: "700"
});

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
 * @return Promise<Buffer> The rendered image as a Buffer
 */
async function generateUserRankGraphic(
  member,
  victimRank,
  punisherRank,
  timesSmacked,
  timesCalled
) {
  templateImage = templateImage || await Canvas.loadImage(templatePath);
  let canvas = Canvas.createCanvas(600, 150);
  let ctx = canvas.getContext("2d");
  ctx.fillStyle = "white";

  // replace the webp format to png so that the package is able to use it
  let avatarURL = member.user.avatarURL()
    .replace(/webp$/, "png") + "?size=256";
  let userAvatar = await Canvas.loadImage(avatarURL);

  // draw the user avatar
  ctx.drawImage(userAvatar, 9, 9, 132, 132);

  // draw the badge frame
  ctx.drawImage(templateImage, 0, 0);

  // draw the username
  ctx.font = "bold 21.1px " + fontFamily;
  ctx.textAlign = "left";
  ctx.fillText(member.displayName, 150, 25, 240);

  // draw the user discriminator
  ctx.font = "13.4px " + fontFamily;
  ctx.fillText("#" + member.user.discriminator, 150, 42);

  // draw the stats
  ctx.font = "bold 16.7px " + fontFamily;
  ctx.textAlign = "center";
  ctx.fillText("#" + victimRank[0], 304, 65);
  ctx.fillText("#" + victimRank[1], 364, 65);
  ctx.fillText("#" + punisherRank[0], 304, 90);
  ctx.fillText("#" + punisherRank[1], 364, 90);
  ctx.fillText(timesSmacked[0], 304, 115);
  ctx.fillText(timesSmacked[1], 364, 115);
  ctx.fillText(timesCalled[0], 304, 140);
  ctx.fillText(timesCalled[1], 364, 140);
  return canvas.toBuffer();
}

module.exports = generateUserRankGraphic;
