import {ApplicationCommandData, GuildMember, MessageAttachment} from "discord.js";

import constructCommand from "../functions/util/constructCommand";
import fetchMemberRanks from "../functions/fetchMemberRanks";
import getRandom from "../functions/util/getRandom";
import replies from "../provider/replies";
import generateUserRankGraphic from "../functions/generateUserRankGraphic";
import GuildCommandInteraction from "../types/GuildCommandInteraction";

/** Ranks command to display a badge of your stats. */
async function ranksCommand(interaction: GuildCommandInteraction) {
  const member = (interaction.options.getMember("user")
    ?? interaction.member) as GuildMember;

  const font = interaction.options.getString("font") ? "Nezuko" : undefined;

  await interaction.deferReply();

  let fetchedRankedMember = await fetchMemberRanks(member, interaction.guild);
  if (!fetchedRankedMember) {
    interaction.editReply(getRandom(replies.noMember)).catch(console.error);
    return;
  }

  let imageBuffer = await generateUserRankGraphic(
    member,
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
    ],
    font
  );

  let attachment = new MessageAttachment(
    imageBuffer,
    `Rank ${fetchedRankedMember.UserName} ${new Date()}.png`
  );
  interaction.editReply({files: [attachment]})
    .catch(console.error);
}

const commandData: ApplicationCommandData = {
  name: "ranks",
  description: "Display your takedown stats in a rank badge",
  options: [
    {
      name: "user",
      description: "Display the rank badge for someone else",
      type: "USER",
      required: false
    },
    {
      name: "font",
      description: "If you're weird, you can overwrite the font here",
      type: "STRING",
      required: false,
      choices: [
        {
          name: "Nezuko",
          value: "Nezuko"
        }
      ]
    }
  ]
}

export default constructCommand(ranksCommand, commandData);
