use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct TimelineOpts {
    /// Tracks file.  These should contain only TM_TRACKS lines from tms_to_csv output
    /// A simple cat my_tele_.csv | grep "TM_TRACS" > my_tele.tracks.csv should do it
    #[structopt(short = "t", long = "track-file")]
    pub track_file: String,

    /// The track you wish to search on.
    #[structopt(short = "m", long = "main-track", default_value = "Main Thread")]
    pub main_track: String,

    /// The track you wish to search on.
    #[structopt(short = "m", long = "context-track", default_value = "Instrumentation")]
    pub context_track: String,

    /// Zones file.  These should contain only TM_ZONES lines from tms_to_csv output
    /// A simple cat my_tele_.csv | grep "TM_ZONE" > my_tele.zones.csv should do it
    #[structopt(short = "z", long = "zone-file")]
    pub zone_file: String,

    /// Specifies the queries to run over the data provided.
    ///
    /// The format of the query file should be the following
    /// ```json
    /// {
    ///     zone_names: [
    ///         <string>, // the name of zones
    ///         ...
    ///     ],
    ///
    ///     // Descriptions of each type found below
    ///     queries: [
    ///         {
    ///             type: "duration",
    ///             between: ["zone_A", "zone_B"], // measures the start time
    ///             start?: true | false, // default = true
    ///             end?: true | false, // default = false
    ///         },
    ///         {
    ///         }
    ///     ]
    /// }
    /// ```
    ///
    /// query-types:
    /// duration:
    /// A typical query.  Used for measuring the duration from the start, or end, of zone A to the
    /// start, or end (respectively) of zone B.  If `start` is true, then it will do the following
    /// equation.  B.start_time - A.start_time.  if `start` is false and `end` isn't provided, or
    /// `end` is true, then A.end_time - B.end_time.
    ///
    /// This assumes that A subsumes B.
    #[structopt(short = "q", long = "query-file")]
    pub query_file: String,
}

