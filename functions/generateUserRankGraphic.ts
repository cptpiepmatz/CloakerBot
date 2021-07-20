import {join} from "path";
import {createCanvas, Image, loadImage, registerFont} from "canvas";
import {GuildMember} from "discord.js";

const templatePath = join(__dirname, "../assets/graphics/user_rank.png");
let templateImage: Image;

// Register the free to use font Roboto and some others
const fontFamily = '"Robot", "Cambria Math", "Segoe UI", sans-serif';

const robotoPath = join(__dirname, "../assets/fonts/Roboto");
registerFont(robotoPath + "Roboto-Regular.ttf", {
  family: "Roboto",
  weight: "400"
});
registerFont(robotoPath + "Roboto-Bold.ttf", {
  family: "Roboto",
  weight: "700"
});

const segoePath = join(__dirname, "../assets/fonts/Segoe/");
registerFont(segoePath + "segoeui.ttf", {
  family: "Segoe UI",
  weight: "400"
});
registerFont(segoePath + "segoeuib.ttf", {
  family: "Segoe UI",
  weight: "700"
});

/**
 * Generates the graphic for the user rank badge.
 * <p>All the pairs represent first the guild value and second the total value.
 * <p>Returns the rendered image as a Buffer.
 *
 * @param member The member to generate the graphic for
 * @param victimRank Pair of victim rank
 * @param punisherRank Pair of punisher rank
 * @param timesSmacked Pair of times smacked
 * @param timesCalled Pair of times called
 */
async function generateUserRankGraphic(
  member: GuildMember,
  victimRank: [number, number],
  punisherRank: [number, number],
  timesSmacked: [number, number],
  timesCalled: [number, number]
): Promise<Buffer> {
  templateImage = templateImage || await loadImage(templatePath);
  let canvas = createCanvas(600, 150);
  let ctx = canvas.getContext("2d");
  ctx.fillStyle = "white";

  // replace the webp format to png so that the package is able to use it
  let avatarURL = member.user.avatarURL()
    ?.replace(/webp$/, "png") + "?size=256";
  let userAvatar = await loadImage(avatarURL);

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
  ctx.fillText(timesSmacked[0].toString(), 304, 115);
  ctx.fillText(timesSmacked[1].toString(), 364, 115);
  ctx.fillText(timesCalled[0].toString(), 304, 140);
  ctx.fillText(timesCalled[1].toString(), 364, 140);
  return canvas.toBuffer();
}

export default generateUserRankGraphic;
