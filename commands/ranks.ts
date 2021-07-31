import {ApplicationCommandData, CommandInteraction, Guild, GuildMember, MessageAttachment} from "discord.js";

import constructCommand from "../functions/util/constructCommand";
import fetchMemberRanks from "../functions/fetchMemberRanks";
import getRandom from "../functions/util/getRandom";
import replies from "../provider/replies";
import generateUserRankGraphic from "../functions/generateUserRankGraphic";

/** Ranks command to display a badge of your stats. */
async function ranksCommand(interaction: CommandInteraction) {
  const member = (interaction.options.getMember("user")
    ?? interaction.member) as GuildMember;
  const guild = interaction.guild as Guild;

  await interaction.defer();

  let fetchedRankedMember = await fetchMemberRanks(member, guild);
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
    ]
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
