use std::time::Duration;

// macro for including sound assets
macro_rules! sa {
    ($name:ident, $file:literal) => {
        pub const $name: (&str, &[u8]) = (
            $file,
            include_bytes!(concat!("../../assets/sounds/", $file))
        );
    };
}

pub mod fails {
    sa!(PAIN1, "fails/pain1.wav");
    sa!(PAIN2, "fails/pain2.wav");
    sa!(PAIN3, "fails/pain3.wav");
    sa!(PAIN4, "fails/pain4.wav");
    sa!(PAIN5, "fails/pain5.wav");
    sa!(PAIN6, "fails/pain6.wav");
    sa!(PAIN7, "fails/pain7.wav");
    sa!(PAIN8, "fails/pain8.wav");
    sa!(PAIN9, "fails/pain9.wav");
    sa!(PAIN10, "fails/pain10.wav");
    sa!(PAIN11, "fails/pain11.wav");
    sa!(PAIN12, "fails/pain12.wav");
    sa!(PAIN13, "fails/pain13.wav");
    sa!(PAIN14, "fails/pain14.wav");

    pub const ALL: &[(&str, &[u8])] = &[
        PAIN1, PAIN2, PAIN3, PAIN4, PAIN5, PAIN6, PAIN7, PAIN8, PAIN9, PAIN10, PAIN11, PAIN12,
        PAIN13, PAIN14
    ];
}

pub mod takedowns {
    sa!(BEAT_THE_SHIT, "takedowns/beat the shit.wav");
    sa!(
        BETTER_LATE_THAN_NEVER,
        "takedowns/better late than never.wav"
    );
    sa!(CHECK_YOUR_CORNERS, "takedowns/check your corners.wav");
    sa!(COME_BACK_FOR_MORE, "takedowns/come back for more.wav");
    sa!(CRY_FOR_MOM, "takedowns/cry for mom.wav");
    sa!(
        DIDNT_FORGET_ABOUT_ME,
        "takedowns/didn't forget about me.wav"
    );
    sa!(DIFFICULTY_TWEAK, "takedowns/difficulty tweak.wav");
    sa!(DRUM_SOLO, "takedowns/drum solo.wav");
    sa!(GO_TO_THE_FORUMS, "takedowns/go to the forums.wav");
    sa!(
        GOTTA_STOP_MEETING_LIKE_THIS,
        "takedowns/gotta stop meeting like this.wav"
    );
    sa!(
        GUESS_YOURE_NOT_AS_GOOD,
        "takedowns/guess you're not as good.wav"
    );
    sa!(
        I_BET_YOU_LET_YOURSELF_GET_BEAT_UP,
        "takedowns/i bet you let yourself get beat up.wav"
    );
    sa!(I_EXPECTED_BETTER, "takedowns/i expected better.wav");
    sa!(IM_GONNA_BEAT_THE, "takedowns/i'm gonna beat the.wav");
    sa!(IM_LATE, "takedowns/i'm late.wav");
    sa!(IVE_GOT_YOUR_DLC, "takedowns/i've got your dlc.wav");
    sa!(
        IVE_GOT_YOUR_SAFEHOUSE,
        "takedowns/i've got your safehouse.wav"
    );
    sa!(MISSED_ME, "takedowns/missed me.wav");
    sa!(NICE_WORKOUT, "takedowns/nice workout.wav");
    sa!(
        NOT_AS_TOUGH_AS_THEY_TOLD_ME,
        "takedowns/not as tough as they told me.wav"
    );
    sa!(NOT_SO_TOUGH_NOW, "takedowns/not so tough now.wav");
    sa!(POINT_OF_NO_RETURN, "takedowns/point of no return.wav");
    sa!(PUT_UP_A_BETTER_FIGHT, "takedowns/put up a better fight.wav");
    sa!(PUTTING_UP_A_FIGHT, "takedowns/putting up a fight.wav");
    sa!(RESISTING_ARREST, "takedowns/resisting arrest.wav");
    sa!(SPEAK_UP, "takedowns/speak up.wav");
    sa!(STOP_HITTING_YOURSELF, "takedowns/stop hitting yourself.wav");
    sa!(THAT_ONES_GONNA_HURT, "takedowns/that one's gonna hurt.wav");
    sa!(THE_SAFEWORD, "takedowns/the safeword.wav");
    sa!(
        THIS_MIGHT_STING_A_LITTLE,
        "takedowns/this might sting a little.wav"
    );
    sa!(TRY_A_LITTLE_HARDER, "takedowns/try a little harder.wav");
    sa!(TURN_YOU_INSIDE_OUT, "takedowns/turn you inside out.wav");
    sa!(
        WE_GOTTA_DO_THIS_MORE_OFTEN,
        "takedowns/we gotta do this more often.wav"
    );
    sa!(
        WEAR_THAT_SHIT_IN_YOUR_PANTS_PROUDLY,
        "takedowns/wear that shit in your pants proudly.wav"
    );
    sa!(WHOS_THE_CLOWN_NOW, "takedowns/who's the clown now.wav");
    sa!(WISHES_DO_COME_TRUE, "takedowns/wishes do come true.wav");
    sa!(WORK_SMARTER, "takedowns/work smarter.wav");
    sa!(YOU_ASKED_FOR_IT, "takedowns/you asked for it.wav");
    sa!(
        YOU_GOT_WHAT_WAS_COMING,
        "takedowns/you got what was coming.wav"
    );

    pub const ALL: &[(&str, &[u8])] = &[
        BEAT_THE_SHIT,
        BETTER_LATE_THAN_NEVER,
        CHECK_YOUR_CORNERS,
        COME_BACK_FOR_MORE,
        CRY_FOR_MOM,
        DIDNT_FORGET_ABOUT_ME,
        DIFFICULTY_TWEAK,
        DRUM_SOLO,
        GO_TO_THE_FORUMS,
        GOTTA_STOP_MEETING_LIKE_THIS,
        GUESS_YOURE_NOT_AS_GOOD,
        I_BET_YOU_LET_YOURSELF_GET_BEAT_UP,
        I_EXPECTED_BETTER,
        IM_GONNA_BEAT_THE,
        IM_LATE,
        IVE_GOT_YOUR_DLC,
        IVE_GOT_YOUR_SAFEHOUSE,
        MISSED_ME,
        NICE_WORKOUT,
        NOT_AS_TOUGH_AS_THEY_TOLD_ME,
        NOT_SO_TOUGH_NOW,
        POINT_OF_NO_RETURN,
        PUT_UP_A_BETTER_FIGHT,
        PUTTING_UP_A_FIGHT,
        RESISTING_ARREST,
        SPEAK_UP,
        STOP_HITTING_YOURSELF,
        THAT_ONES_GONNA_HURT,
        THE_SAFEWORD,
        THIS_MIGHT_STING_A_LITTLE,
        TRY_A_LITTLE_HARDER,
        TURN_YOU_INSIDE_OUT,
        WE_GOTTA_DO_THIS_MORE_OFTEN,
        WEAR_THAT_SHIT_IN_YOUR_PANTS_PROUDLY,
        WHOS_THE_CLOWN_NOW,
        WISHES_DO_COME_TRUE,
        WORK_SMARTER,
        YOU_ASKED_FOR_IT,
        YOU_GOT_WHAT_WAS_COMING
    ];
}

// TODO: get more precise duration
pub const WULULULULU_LEN: Duration = Duration::from_secs(5);
pub const WULULULULU: (&str, &[u8]) = (
    "WULULULULU.wav",
    include_bytes!("../../assets/sounds/WULULULULU.wav")
);
