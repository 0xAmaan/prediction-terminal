// ============================================================================
// TEAM NAME ABBREVIATIONS
// ============================================================================

const abbrevMap: Record<string, string> = {
  // NFL Teams
  Seahawks: "SEA",
  Falcons: "ATL",
  Colts: "IND",
  Jaguars: "JAX",
  Patriots: "NE",
  Bills: "BUF",
  Dolphins: "MIA",
  Jets: "NYJ",
  Ravens: "BAL",
  Steelers: "PIT",
  Browns: "CLE",
  Bengals: "CIN",
  Texans: "HOU",
  Titans: "TEN",
  Chiefs: "KC",
  Raiders: "LV",
  Chargers: "LAC",
  Broncos: "DEN",
  Cowboys: "DAL",
  Eagles: "PHI",
  Giants: "NYG",
  Commanders: "WAS",
  Packers: "GB",
  Bears: "CHI",
  Vikings: "MIN",
  Lions: "DET",
  Buccaneers: "TB",
  Saints: "NO",
  Panthers: "CAR",
  "49ers": "SF",
  Rams: "LAR",
  Cardinals: "ARI",

  // NBA Teams
  Celtics: "BOS",
  Raptors: "TOR",
  Lakers: "LAL",
  Warriors: "GSW",
  Clippers: "LAC",
  Nets: "BKN",
  Knicks: "NYK",
  "76ers": "PHI",
  Heat: "MIA",
  Magic: "ORL",
  Hawks: "ATL",
  Hornets: "CHA",
  Wizards: "WAS",
  Bulls: "CHI",
  Cavaliers: "CLE",
  Pistons: "DET",
  Pacers: "IND",
  Bucks: "MIL",
  Timberwolves: "MIN",
  Thunder: "OKC",
  Pelicans: "NOP",
  Mavericks: "DAL",
  Rockets: "HOU",
  Grizzlies: "MEM",
  Spurs: "SAS",
  Suns: "PHX",
  Jazz: "UTA",
  Nuggets: "DEN",
  "Trail Blazers": "POR",
  Kings: "SAC",

  // Soccer/La Liga
  "Real Madrid": "RMA",
  "Celta de Vigo": "CEL",
  Barcelona: "BAR",
  "Atletico Madrid": "ATM",
  Sevilla: "SEV",
  Valencia: "VAL",
  "Athletic Bilbao": "ATH",
  Villarreal: "VIL",
  "Real Sociedad": "RSO",
  "Real Betis": "BET",

  // Premier League
  "Manchester United": "MUN",
  "Manchester City": "MCI",
  Liverpool: "LIV",
  Chelsea: "CHE",
  Arsenal: "ARS",
  Tottenham: "TOT",
  "Newcastle United": "NEW",
  "West Ham": "WHU",
  "Aston Villa": "AVL",
  Brighton: "BHA",
  Everton: "EVE",

  // eSports
  G2: "G2",
  "Team Falcons": "TF",
  Fnatic: "FNC",
  "Team Liquid": "TL",
  Cloud9: "C9",
  T1: "T1",
  "Gen.G": "GEN",
  DRX: "DRX",
  NaVi: "NAVI",
  Vitality: "VIT",
  FaZe: "FAZE",
  Heroic: "HERO",
  MOUZ: "MOUZ",
  NIP: "NIP",
  Astralis: "AST",
};

export const getTeamAbbrev = (name: string): string => {
  // Check direct match first
  if (abbrevMap[name]) {
    return abbrevMap[name];
  }

  // Check partial matches
  for (const [fullName, abbrev] of Object.entries(abbrevMap)) {
    if (name.toLowerCase().includes(fullName.toLowerCase())) {
      return abbrev;
    }
  }

  // Fallback: first 3 characters uppercase
  return name.slice(0, 3).toUpperCase();
};
