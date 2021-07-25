enum GuildStats {
  // How often a bot was taken down on a guild
  BOTS_TAKEN_DOWN = "botsTakenDown",

  // How often people commanded a takedown on a guild
  COMMANDED_TAKEDOWNS = "commandedTakedowns",

  // How often commanded takedown was denied on a guild
  DENIED_TAKEDOWNS = "deniedTakedowns",

  // How often someone was randomly taken down on a guild
  RANDOM_TAKEDOWNS = "randomTakedowns",

  // How often takedowns failed on a guild
  TAKEDOWNS_FAILED = "takedownsFailed",

  // How often takedowns were successful on a guild
  TAKEDOWNS_SUCCESS = "takedownsSuccess"
}

enum UserStats {
  // How often a user commanded a takedown per guild
  TIMES_COMMANDED_TAKEDOWN = "timesCommandedTakedown",

  // How often a user was denied to command a takedown per guild
  TIMES_DENIED_COMMAND_TAKEDOWN = "timesDeniedCommandedTakedown",

  // How often a user entered an incorrect command
  TIMES_INCORRECT_COMMAND = "timesIncorrectCommand",

  // How often a user was taken down per guild
  TIMES_TAKEN_DOWN = "timesTakenDown"
}

/** Enum for the stats to track */
const Stats = {
  ...GuildStats,
  ...UserStats
}

type Stats = GuildStats | UserStats

export default Stats;
export {GuildStats, UserStats}
