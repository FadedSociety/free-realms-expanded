// Treasure War: the movie the server's stand-in game plays through (frame 1 of
// game_treasurewar.gfx, built by build.py). Original work for this project, AGPL-3.0-or-later.
//
// It draws everything itself (no library symbols), takes card and opponent pictures from the
// client's own files by name, and takes every prompt and name from the client's locale by text
// id (none is written here). The
// server owns the rules; this only shows what it is told and reports clicks.
//
// Host contract (the client's MiniGameFlash window): text out is
// fscommand("FR_event", "SendMsg( \"...\")"), text in is _root.OnMessage(payload), and a
// locale string is asked for with StringById(id) and arrives in _root.OnString(id, text).
//
// Server text, tab separated (src/treasure_war_table.rs):
//   TW_Names <name id> <picture>      the opponent; 0 0 is another player
//   TW_Hand <card>...                 card = art id,name id,colour,power
//   TW_Board <mine> <theirs>          one letter a treasure: d face down, t taken
//   TW_Duel <mine> <theirs> <result>  - none, ? face down; 0 none, 1 won, 2 lost, 3 tie
//   TW_Flip <side> <slot> <card> <taken>   side 0 is my treasure, 1 theirs
//   TW_Phase <name> <text id> <mine taken> <theirs taken>
//            name: choose, wait, attack, defend, waiting, won, lost
// Out: TW_Ready, TW_Play <hand slot>, TW_Take <treasure slot>, FRServer_GameOver.

var STAGE_W = 800;
var STAGE_H = 600;
var CARD_W = 108;
var CARD_H = 146;
var COLOURS = [0xC0392B, 0x2E9E4F, 0x2A7FC9];
var COLOUR_NAMES = ["R", "G", "B"];
var DUEL_PAUSE = 1700;
var FLIP_PAUSE = 1900;

var strings = {};
var wanted = {};
var queue = [];
var busyUntil = 0;
var phase = "";
var heard = false;
var overSent = false;
var layers = {};

function send(text) {
   fscommand("FR_event", "SendMsg( \"" + text + "\")");
}

function format(size, colour, align) {
   var f = new TextFormat();
   f.font = "$MainFont";
   f.size = size;
   f.color = colour;
   f.align = align;
   f.bold = true;
   return f;
}

function label(parent, name, x, y, w, h, size, colour, align) {
   parent.createTextField(name, parent.getNextHighestDepth(), x, y, w, h);
   var tf = parent[name];
   tf.selectable = false;
   tf.multiline = true;
   tf.wordWrap = true;
   tf.setNewTextFormat(format(size, colour, align));
   tf.fmt = format(size, colour, align);
   return tf;
}

function setText(tf, text) {
   tf.text = text;
   tf.setTextFormat(tf.fmt);
}

// Puts locale string `id` into `tf`, now or when the client answers.
function setString(tf, id, fallback) {
   var key = String(id);
   if (strings[key] != undefined) {
      setText(tf, strings[key]);
      return;
   }
   setText(tf, fallback);
   if (Number(id) <= 0) {
      return;
   }
   if (wanted[key] == undefined) {
      wanted[key] = [];
      fscommand("FR_event", "StringById(" + key + ")");
   }
   wanted[key].push(tf);
}

_root.OnString = function(id, text) {
   var key = String(id);
   if (text == undefined || text == "") {
      return;
   }
   strings[key] = text;
   var list = wanted[key];
   if (list != undefined) {
      var i = 0;
      while (i < list.length) {
         setText(list[i], text);
         i = i + 1;
      }
      delete wanted[key];
   }
};

function box(mc, x, y, w, h, fill, line, alpha) {
   mc.lineStyle(2, line, 100);
   mc.beginFill(fill, alpha);
   mc.moveTo(x, y);
   mc.lineTo(x + w, y);
   mc.lineTo(x + w, y + h);
   mc.lineTo(x, y + h);
   mc.lineTo(x, y);
   mc.endFill();
}

function layer(name, depth) {
   if (layers[name] != undefined) {
      layers[name].removeMovieClip();
   }
   layers[name] = _root.createEmptyMovieClip(name, depth);
   return layers[name];
}

// The client's own picture `file`, fitted into w by h once it has loaded.
function picture(parent, name, file, x, y, w, h) {
   var holder = parent.createEmptyMovieClip(name, parent.getNextHighestDepth());
   holder._x = x;
   holder._y = y;
   var loader = new MovieClipLoader();
   var listener = new Object();
   listener.onLoadInit = function(target) {
      target._width = w;
      target._height = h;
   };
   loader.addListener(listener);
   loader.loadClip(file, holder);
   return holder;
}

function parseCard(token) {
   var p = token.split(",");
   if (p.length < 4) {
      return null;
   }
   return {art:p[0], nameId:p[1], colour:Number(p[2]), power:p[3]};
}

// A face-up card, a face-down one (card null), or an empty slot (card undefined).
function drawCard(parent, name, x, y, card) {
   var mc = parent.createEmptyMovieClip(name, parent.getNextHighestDepth());
   mc._x = x;
   mc._y = y;
   if (card == undefined) {
      box(mc, 0, 0, CARD_W, CARD_H, 0x000000, 0x8A7A55, 20);
      return mc;
   }
   if (card == null) {
      box(mc, 0, 0, CARD_W, CARD_H, 0x5B3A1E, 0xE0C070, 100);
      box(mc, 10, 10, CARD_W - 20, CARD_H - 20, 0x7A4E28, 0xE0C070, 100);
      setText(label(mc, "q", 0, 46, CARD_W, 50, 34, 0xE0C070, "center"), "?");
      return mc;
   }
   box(mc, 0, 0, CARD_W, CARD_H, COLOURS[card.colour], 0xFFFFFF, 100);
   box(mc, 6, 30, CARD_W - 12, 78, 0x1B1B1B, 0xFFFFFF, 100);
   picture(mc, "art", card.art + ".dds", 8, 32, CARD_W - 16, 74);
   setText(label(mc, "power", 4, 2, 44, 26, 18, 0xFFFFFF, "left"), COLOUR_NAMES[card.colour] + card.power);
   setString(label(mc, "title", 4, 110, CARD_W - 8, 36, 11, 0xFFFFFF, "center"), card.nameId, "");
   return mc;
}

function drawChest(parent, name, x, y) {
   var mc = parent.createEmptyMovieClip(name, parent.getNextHighestDepth());
   mc._x = x;
   mc._y = y;
   box(mc, 0, 0, 78, 100, 0x000000, 0x8A7A55, 25);
   box(mc, 14, 50, 50, 30, 0x7A4E28, 0xE0C070, 100);
   box(mc, 14, 28, 50, 12, 0x5B3A1E, 0xE0C070, 100);
   return mc;
}

function drawBack(parent, name, x, y) {
   var mc = parent.createEmptyMovieClip(name, parent.getNextHighestDepth());
   mc._x = x;
   mc._y = y;
   box(mc, 0, 0, 78, 100, 0x5B3A1E, 0xE0C070, 100);
   box(mc, 8, 8, 62, 84, 0x7A4E28, 0xE0C070, 100);
   return mc;
}

function showBackdrop() {
   var mc = layer("backdrop", 1);
   box(mc, 0, 0, STAGE_W, STAGE_H, 0x1F3A2B, 0xE0C070, 100);
   box(mc, 130, 70, 540, 300, 0x17402B, 0x8A7A55, 100);
   setString(label(mc, "title", 200, 8, 400, 34, 24, 0xE0C070, "center"), 6949, "Treasure War");
}

function showNames(nameId, image) {
   var mc = layer("names", 2);
   var tf = label(mc, "them", 500, 44, 220, 24, 15, 0xFFFFFF, "right");
   if (Number(nameId) > 0) {
      setString(tf, nameId, "");
      picture(mc, "face", "campaignImage_" + image + ".dds", 728, 10, 60, 60);
   } else {
      setText(tf, "");
   }
}

function showHand(tokens) {
   var mc = layer("hand", 6);
   var count = tokens.length - 1;
   var left = (STAGE_W - count * (CARD_W + 14) + 14) / 2;
   var i = 1;
   while (i < tokens.length) {
      var card = parseCard(tokens[i]);
      if (card != null) {
         var clip = drawCard(mc, "card" + i, left + (i - 1) * (CARD_W + 14), 440, card);
         clip.slot = i - 1;
         clip.onRollOver = function() {
            this._y = 424;
         };
         clip.onRollOut = function() {
            this._y = 440;
         };
         clip.onRelease = function() {
            if (phase == "choose") {
               phase = "sent";
               send("TW_Play\t" + this.slot);
            }
         };
      }
      i = i + 1;
   }
}

function showBoard(mine, theirs) {
   var mc = layer("board", 4);
   var i = 0;
   while (i < mine.length) {
      if (mine.charAt(i) == "t") {
         drawChest(mc, "mine" + i, 26, 84 + i * 86);
      } else {
         drawBack(mc, "mine" + i, 26, 84 + i * 86);
      }
      i = i + 1;
   }
   i = 0;
   while (i < theirs.length) {
      if (theirs.charAt(i) == "t") {
         drawChest(mc, "theirs" + i, 696, 84 + i * 86);
      } else {
         var clip = drawBack(mc, "theirs" + i, 696, 84 + i * 86);
         clip.slot = i;
         clip.onRelease = function() {
            if (phase == "attack") {
               phase = "sent";
               send("TW_Take\t" + this.slot);
            }
         };
      }
      i = i + 1;
   }
}

function duelSide(mc, name, x, token) {
   if (token == "-") {
      drawCard(mc, name, x, 110, undefined);
   } else if (token == "?") {
      drawCard(mc, name, x, 110, null);
   } else {
      drawCard(mc, name, x, 110, parseCard(token));
   }
}

function showDuel(mine, theirs, result) {
   var mc = layer("duel", 5);
   duelSide(mc, "mine", 262, mine);
   duelSide(mc, "theirs", 430, theirs);
   var words = ["", "WIN", "LOSE", "TIE"];
   var tints = [0xFFFFFF, 0x7CFC7C, 0xFF8A7A, 0xFFE27A];
   setText(label(mc, "result", 200, 262, 400, 40, 26, tints[Number(result)], "center"), words[Number(result)]);
}

function showFlip(side, slot, token, taken) {
   var mc = layer("flip", 8);
   var x = (Number(side) == 1) ? 580 : 122;
   drawCard(mc, "card", x, 84 + Number(slot) * 60, parseCard(token));
   var word = (Number(taken) == 1) ? "TAKEN" : "SAFE";
   var tint = ((Number(taken) == 1) == (Number(side) == 1)) ? 0x7CFC7C : 0xFF8A7A;
   setText(label(mc, "word", 200, 262, 400, 40, 26, tint, "center"), word);
}

function showPhase(name, textId, mine, theirs) {
   phase = name;
   if (layers.flip != undefined) {
      layers.flip.removeMovieClip();
      delete layers.flip;
   }
   var mc = layer("prompt", 7);
   box(mc, 180, 380, 440, 44, 0x000000, 0xE0C070, 60);
   setString(label(mc, "text", 186, 390, 428, 30, 15, 0xFFFFFF, "center"), textId, "");
   setText(label(mc, "score", 250, 40, 300, 24, 14, 0xE0C070, "center"), mine + " - " + theirs);
   if (name == "won" || name == "lost") {
      box(mc, 340, 300, 120, 40, 0x7A4E28, 0xE0C070, 100);
      setString(label(mc, "exit", 340, 308, 120, 28, 16, 0xFFFFFF, "center"), 433310, "Exit");
      var button = mc.createEmptyMovieClip("hit", mc.getNextHighestDepth());
      box(button, 340, 300, 120, 40, 0xFFFFFF, 0xFFFFFF, 0);
      button.onRelease = function() {
         gameOver();
      };
      busyUntil = getTimer() + 6000;
      queue.push("TW_Over");
   }
}

function gameOver() {
   if (!overSent) {
      overSent = true;
      send("FRServer_GameOver");
   }
}

// One server message; returns how long to hold the next one back.
function handle(payload) {
   var f = payload.split("\t");
   var name = f[0];
   if (name == "TW_Names") {
      showNames(f[1], f[2]);
   } else if (name == "TW_Hand") {
      showHand(f);
   } else if (name == "TW_Board") {
      showBoard(f[1], f[2]);
   } else if (name == "TW_Duel") {
      showDuel(f[1], f[2], f[3]);
      if (Number(f[3]) > 0) {
         return DUEL_PAUSE;
      }
   } else if (name == "TW_Flip") {
      showFlip(f[1], f[2], f[3], f[4]);
      return FLIP_PAUSE;
   } else if (name == "TW_Phase") {
      showPhase(f[1], f[2], f[3], f[4]);
   } else if (name == "TW_Over") {
      gameOver();
   }
   return 0;
}

_root.OnMessage = function(payload) {
   if (payload == undefined) {
      return;
   }
   heard = true;
   queue.push(String(payload));
};

_root.getResizeInfo = function() {
   fscommand("FR_event", "Resize( " + STAGE_W + "," + STAGE_H + ")");
};
_root.OnTutorialClosed = function() {
   send("TW_Ready");
};
_root.QuitGame = function() {
};
_root.OnEndGame = function() {
};
_root.PauseGame = function() {
};
_root.ResumeGame = function() {
};

var ticks = 0;
_root.onEnterFrame = function() {
   ticks = ticks + 1;
   // Until the server has spoken, ask for the table about every two seconds.
   if (!heard && ticks % 60 == 30) {
      send("TW_Ready");
   }
   while (queue.length > 0 && getTimer() >= busyUntil) {
      var pause = handle(String(queue.shift()));
      if (pause > 0) {
         busyUntil = getTimer() + pause;
      }
   }
};

Stage.scaleMode = "exactFit";
showBackdrop();
fscommand("FR_event", "Resize( " + STAGE_W + "," + STAGE_H + ")");
fscommand("FR_event", "ReadyToStartGame()");
