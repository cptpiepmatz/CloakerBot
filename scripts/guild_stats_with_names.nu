# use the names.json to make guild_stats.json use names instead
let data_dir = $env.FILE_PWD | path dirname | path join "data"
let names = open ($data_dir | path join "names.json")
open ($data_dir | path join "guild_stats.json") | items {
    |guild, stats| {($names | get $guild): (
        $stats | get takedowns | {takedowns: ($in | items {
            |caller, targets| {($names | get $caller): (
                $targets | items {
                    |target, count| {($names | get $target): $count}
                } | get 0
            )}
        } | get 0)}
    )}
} | get 0
