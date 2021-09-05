import SendableText from "../types/SendableText";
import {MessageActionRow, MessageButton, MessageEmbed} from "discord.js";
import colors from "../provider/colors";

const timestamp = "2021-08-29T11:42:00";

const embed = new MessageEmbed()
  .setTitle(":tada: Update 2.0.0 :tada:")
  .setColor(colors.primary)
  .setAuthor(
    "Piepmatz#0666 (Creator of the bot)",
    "https://cdn.discordapp.com/avatars/137280289708834816/b720211b15b5baafd5450786c622a220.webp?size=128",
    "https://cptpiepmatz.de"
  )
  .setDescription(
    "The bot just updated to Version 2.0.0 and this comes with some quite big changes. \n​"
  )
  .setFooter("Automatically sent message - AutoMessage: 0")
  .setThumbnail(
    "https://cdn.discordapp.com/avatars/772259037580361729/655bc7f00ca5f8598cabf4781f5cfb80.webp?size=512"
  )
  .addField(
    ":arrow_up: Discord API v9 compatible",
    "Discord updates their api relative regularly and therefore bots " +
    "need too. So I did."
  )
  .addField(
    ":mega: Slash Commands",
    "Discord introduced a new feature for bots to use. " +
    "It's called slash commands. " +
    "They enable you to use commands starting with a \"/\". " +
    "I used something like this before but now it's official. \n" +
    "Also some commands changed a bit but that should be quite easy to adopt " +
    "with the command menu." +
    "\n[A blog entry about them, if you care.]" +
    "(https://blog.discord.com/slash-commands-are-here-8db0a385d9e6)"
  )
  .addField(
    ":warning: Danger Zones",
    "I got a request that mentioned voice channels that you will be " +
    "automatically kicked from. " +
    "Probably funny, I don't know."
  )
  .addField(
    ":mirror: New avatar",
    "I got the bot a new avatar to kinda party the update. " +
    "Hope you like it. " +
    "If you want the shovel back, tell me."
  )
  .addField(
    ":mailbox: New persistent invite link",
    "Since my bot updates from time to time, I made a link that also " +
    "updates if it needs to. \n" +
    "https://cptpiepmatz.de/cloaker-bot/invite-to-server"
  )
  .addField("​", "​")
  .addField(
    ":question: Need help, check my Discord server",
    "Since public release I have a discord server if you have any kind " +
    "of questions, I try to respond quickly on it. " +
    "Maybe you wanna come around. \n" +
    "[Join the server.](https://cptpiepmatz.de/cloaker-bot/join-server)\n​",
    true
  )
  .addField(
    ":dollar: Support me",
    "Also If you wanna support me in any way, feel free to join the " +
    "discord server or " +
    "[buy me a :stuffed_flatbread:](https://www.buymeacoffee.com/cptpiepmatz). " +
    "\n​",
    true
  )
  .setTimestamp(new Date(timestamp))


const content =
  "Hey,\nit seems that you're an owner of a discord server that uses this bot. " +
  "\n\nPlease don't get annoyed just yet, these messages will be sent rarely " +
  "and you will be able to opt out of them.\n​";

const unSubButton = new MessageButton()
  .setCustomId("dm-unsub")
  .setLabel("Unsubscribe from these kind of messages")
  .setStyle("DANGER");

const update200: SendableText = {
  content: content,
  embeds: [embed],
  components: [new MessageActionRow().addComponents(unSubButton)]
};

export default update200;
