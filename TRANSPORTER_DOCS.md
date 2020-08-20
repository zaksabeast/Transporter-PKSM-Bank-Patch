# Transporter Documentation

The goal of this document is to describe a bit about how Transporter works and how the PKSM Bank patch affects Transporter.

Please keep in mind there may be inaccuracies while reading this. This document aims to simplify some of the actual workings and leave out smaller details. Offsets will be provided throughout the document in case anyone is interested in digging deeper.

If you use this document to help make any patches, crediting me would be appreciated.

## High level overview

Transporter is basically a giant state machine made up of smaller state machines.

One function handles the overall state of Transporter, and the app continuously runs that same function over and over

At a high level, this looks something like:

```c++
void show_intro() {
  // run logic
}

void transfer_pokemon() {
  // run logic
}

u32 handle_state(u32 main_state) {
  // Continue running logic for the current state by default
  u32 next_state = main_state;

  switch (main_state) {
    case 0:
      show_intro();
      next_state = 1;
      break;
    case 1:
      transfer_pokemon();
      next_state = 2;
      break;
  }

  return next_state;
}

int main() {
  u32 main_state = 0;

  while (main_state != 2) {
    main_state = handle_state(main_state);
  }

  return 0;
}
```

Every case being handled has a state of its own (hence the previously mentioned smaller state machines). Many of the smaller state machines, such as any waiting for user input, won't be finished immediately and need to be waited on before the overall app state continues.

Each of the mini states returns a boolean that describes if they are completed. If `true`, the overall state will proceed to the next state.

```c++
bool show_intro(u32* state) {
  // Continue showing intro by default
  bool is_finished_showing_intro = false;

  switch(*state) {
    case 0:
      bool is_finished_animating = render_animation_frame();
      if (is_finished_animating) {
        *state = 1;
      }
      // If the animation isn't finished, don't change the state
      // this means state == 0 and will continue falling into this case
      break;
    case 1:
      bool has_user_pressed_button = get_user_input();
      if (has_user_pressed_button) {
        // Have overall state proceed to the next state
        is_finished_showing_intro = true;
      }
      break;
  }

  return is_finished_showing_intro;
}

bool transfer_pokemon(u32* state) {
  // run logic
  return is_finished_transferring_pokemon;
}

enum AppState {
  SHOW_INTRO,
  TRANSFER_POKEMON,
  COMPLETED,
}

AppState handle_overall_app_state(AppState main_state, u32* mini_state) {
  // Continue running logic for the current state by default
  u32 next_state = main_state;

  switch (main_state) {
    case SHOW_INTRO:
      if (show_intro(mini_state)) {
        next_state = TRANSFER_POKEMON;
      }
      break;
    case TRANSFER_POKEMON:
      if (transfer_pokemon(mini_state)) {
        next_state = COMPLETED;
      }
      break;
  }

  return next_state;
}

int main() {
  AppState main_state = SHOW_INTRO;
  u32 mini_state = 0;

  while (main_state != 2) {
    main_state = handle_overall_app_state(main_state, &mini_state);
  }

  return 0;
}
```

Each mini state returns a boolean describing if it's finished running, but this doesn't help determine whether the mini state failed or not. For example, how would we know if a Pokemon failed to transfer? For this reason, each mini state also sets a finish status to help determine how it completed.

In addition to this, the overall app state doesn't actually choose its next state. Another function, `get_next_state` below, determines what the next state will be based on the current state and finish status.

```c++
bool show_intro(u32* state, u32* finish_status) {
  // run logic
  return is_finished_showing_intro;
}

enum TransferPokemonState {
  READ_POKEMON_FROM_GAME,
  SEND_POKEMON_TO_SERVER,
  FAILED,
  SUCCEEDED,
}

bool transfer_pokemon(TransferPokemonState* state, u32* finish_status) {
  bool is_finished_transferring_pokemon = false;

  // run logic
  switch(*state) {
    case READ_POKEMON_FROM_GAME:
      bool is_success = read_pokemon_from_game();
      if (is_success) {
        *state = SEND_POKEMON_TO_SERVER;
      } else {
        *state = FAILED;
      }
      break;
    case SEND_POKEMON_TO_SERVER:
      bool is_success = read_pokemon_from_game();
      if (is_success) {
        *state = SUCCEEDED;
      } else {
        *state = FAILED;
      }
      break;
    case FAILED:
      is_finished_transferring_pokemon = true;
      *finish_status = 1;
      break;
    case SUCCEEDED:
      *finish_status = 0;
      is_finished_transferring_pokemon = true;
      break;
  }

  return is_finished_transferring_pokemon;
}

enum AppState {
  SHOW_INTRO,
  TRANSFER_POKEMON,
  SHOW_ERROR,
  COMPLETED,
}

AppState get_next_state(AppState main_state, u32 finish_status) {
  switch (main_state) {
    case SHOW_INTRO:
      return TRANSFER_POKEMON;
    case TRANSFER_POKEMON:
      if (finish_status == 0) {
        return COMPLETED;
      } else {
        return SHOW_ERROR;
      }
    case SHOW_ERROR:
      return SHOW_INTRO;
    default:
      return SHOW_INTRO;
  }
}

bool handle_overall_app_state(AppState main_state, u32* mini_state, u32* finish_status) {
  bool has_finished_state = false;

  switch (main_state) {
    case SHOW_INTRO:
      has_finished_state = show_intro(mini_state, finish_status);
      break;
    case TRANSFER_POKEMON:
      has_finished_state = transfer_pokemon((TransferPokemonState*) mini_state, finish_status);
      break;
    case SHOW_ERROR:
      // show error
      break;
  }

  return has_finished_state;
}

int main() {
  AppState main_state = SHOW_INTRO;
  u32 mini_state = 0;
  u32 finish_status = 0;

  while (main_state != 2) {
    if (handle_overall_app_state(main_state, &mini_state, &finish_status)) {
      main_state = get_next_state(main_state, finish_status);
    }
  }

  return 0;
}
```

## App states

These are the different states the overall app has. There are 19 total states, and there are 12 that are known:

| State name                        | Value | Description                                                                                              |
| --------------------------------- | ----- | -------------------------------------------------------------------------------------------------------- |
| GAME_LOAD                         | 0x0   | This is only run when the game first starts and presumably sets things up for the app                    |
| SELECT_LANGUAGE                   | 0x1   | Shows the language selection menu                                                                        |
| HOME_MENU                         | 0x2   | This renders the home menu and waits for the user to press a button input                                |
| CONNECT_ONLINE                    | 0x3   | This occurs after a user selects a game, and begins connecting to Nintendo's servers                     |
| TRANSFER_POKEMON                  | 0x6   | When a user is prompted to transfer Pokemon, this runs if the user accepts                               |
| SHOW_GAMES                        | 0x8   | This shows a list of games the user can transport from                                                   |
| GET_POKEMON                       | 0x9   | This handles reading Pokemon, legality checking, and converting to EKX                                   |
| CHECK_IF_USER_CAN_TRANSFER        | 0xB   | Checks if the user has Pokemon in their Bank's Transport box (and possibly other checks)                 |
| ASK_TO_TRANSFER                   | 0xC   | Asks if the user if they want to transfer their Pokemon                                                  |
| CLEANUP_PREVIOUS_EARLY_DISCONNECT | 0xE   | This runs on the next Transporter use if there was a problem communicating with Bank on the previous use |
| NO_TRANSFER                       | 0xF   | This runs any time Transporter ends a session without transferring Pokemon                               |
| DISCONNECT_ONLINE                 | 0x10  | Disconnects from online services                                                                         |
| SAVE_SETTINGS                     | 0x11  | Saves the Transporter language selection, and possibly other things                                      |

A healthy flow is:

```
HOME_MENU -> 0x7 -> SHOW_GAMES -> CONNECT_ONLINE -> 0x5 -> GET_POKEMON -> 0xA -> 0xD -> CHECK_IF_USER_CAN_TRANSFER -> ASK_TO_TRANSFER -> TRANSFER_POKEMON -> DISCONNECT_ONLINE -> HOME_MENU
```

Transporter always starts and ends on the home menu.

## Patching the game

The PKSM Bank patch has two primary things it needs to do:

- Check if the user can transfer Pokemon (does the PKSM Bank file exist? Is it valid? Are there any Pokemon in the first box?)
- Transfer Pokemon to the Bank file

So the patch needs to hook into `CHECK_IF_USER_CAN_TRANSFER` and `TRANSFER_POKEMON`.

Hooking into `CHECK_IF_USER_CAN_TRANSFER` allows showing the normal messages a user would see if Transporter were talking to the real bank.

### Modifying the flow of execution

Every state handler in Transporter uses switch statements. When compiled, switch statements will often become jump tables. A jump table is a list of addresses where the execution will jump to. The value given to the switch statement can be used as an index of the jump table.

For example:

```c++
// Jump table:
// 0x190000 (case/index 0 - handle_case_0)
// 0x190020 (case/index 1 - handle_case_1)
// 0x190028 (case/index 2 - handle_case_2)

// Switch statement
switch (state) {
  case 0:
    handle_case_0();
    break;
  case 1:
    handle_case_1();
  case 2:
    handle_case_2();
}
```

In an app like Transporter that relies heavily on switch statements, it's easy to patch the app flow to be whatever we want: just modify the jump tables to jump to a new place of execution.

For example:

```c++
// Jump table:
// 0x200000 (case/index 0 - our_custom_code)
// 0x190020 (case/index 1 - handle_case_1)
// 0x190028 (case/index 2 - handle_case_2)

// Switch statement
switch (state) {
  case 0:
    our_custom_code(); // handle_case_0();
    break;
  case 1:
    handle_case_1();
  case 2:
    handle_case_2();
}
```

Another way to modify the flow is by jumping to custom code from whatever offset the switch statement jump table has:

```c++
// Jump table:
// 0x190000 (case/index 0 - handle_case_0)
// 0x190020 (case/index 1 - handle_case_1)
// 0x190028 (case/index 2 - handle_case_2)

// Switch statement
switch (state) {
  case 0:
    out_custom_code(); // modify this to jump to custom code (`b <offset>`)
    break;
  case 1:
    handle_case_1();
  case 2:
    handle_case_2();
}
```

When building 3ds patches, I'll often use Luma's GDB stub to modify the game live to test and try things out. For some reason, I received a GDB error when modifying a number of Transporter's jump tables using Luma's GDB stub, so I opted for regular jumps. Using regular jumps allows anyone to recreate the patch using GDB and play around with it live as well.

### Patched functions

This is a list of the different functions handy for patching Transporter:

| Name                       | Location         | Reason for patching                                           |
| -------------------------- | ---------------- | ------------------------------------------------------------- |
| handle_overall_app_state   | .text + 0x19cee4 | Skip/replace any time a state is handled                      |
| get_next_state             | .text + 0x242ba0 | Change the next state of any state for a specific finish code |
| check_if_user_can_transfer | .text + 0x248c68 | Replace with custom checks                                    |
| transfer_pokemon           | .text + 0x24a0c4 | Replace with however you'd like to dump EKXs                  |

The PKSM Bank patch modifies `check_if_user_can_transfer` to check if the PKSM Bank file exists and if it's valid.

`transfer_pokemon` is responsible for removing Pokemon from the original game and sending the Pokemon to Bank. The PKSM Bank patch modifies `transfer_pokemon` to still remove Pokemon from the game, but skips saving Pokemon to Bank.

Any time Transporter does not send Pokemon, it must go through the `DISCONNECT_ONLINE` state. If not, opening Bank will alert the user they must go back into Transporter to clean up a mishandled transfer. This is when the state `CLEANUP_PREVIOUS_EARLY_DISCONNECT` is used. This is not an ideal user experience.

The PKSM Bank patch modifies `get_next_state` to always return `DISCONNECT_ONLINE` as the next state after `TRANSFER_POKEMON`. Even though Transporter sends Pokemon to PKSM, it's effectively telling the Bank servers that no transfer took place.

### Jump tables

This can be helpful for exploring Transporter and making more patches.

The jump table of handle_overall_app_state is located at .text + 0x19cf00 and has these jumps:

| Case #    | State Name                        | Offset           | Jump offset      |
| --------- | --------------------------------- | ---------------- | ---------------- |
| case 0x0  | GAME_LOAD                         | .text + 0x19cf00 | .text + 0x19cf4c |
| case 0x1  | SELECT_LANGUAGE                   | .text + 0x19cf04 | .text + 0x19cf8c |
| case 0x2  | HOME_MENU                         | .text + 0x19cf08 | .text + 0x19cfd4 |
| case 0x3  | CONNECT_ONLINE                    | .text + 0x19cf0c | .text + 0x19d020 |
| case 0x4  | _unnamed_                         | .text + 0x19cf10 | .text + 0x19d104 |
| case 0x5  | _unnamed_                         | .text + 0x19cf14 | .text + 0x19d154 |
| case 0x6  | TRANSFER_POKEMON                  | .text + 0x19cf18 | .text + 0x19d1a4 |
| case 0x7  | _unnamed_                         | .text + 0x19cf1c | .text + 0x19d1fc |
| case 0x8  | SHOW_GAMES                        | .text + 0x19cf20 | .text + 0x19d23c |
| case 0x9  | GET_POKEMON                       | .text + 0x19cf24 | .text + 0x19d2c8 |
| case 0xA  | _unnamed_                         | .text + 0x19cf28 | .text + 0x19d310 |
| case 0xB  | CHECK_IF_USER_CAN_TRANSFER        | .text + 0x19cf2c | .text + 0x19d360 |
| case 0xC  | ASK_TO_TRANSFER                   | .text + 0x19cf30 | .text + 0x19d3b4 |
| case 0xD  | _unnamed_                         | .text + 0x19cf34 | .text + 0x19d408 |
| case 0xE  | CLEANUP_PREVIOUS_EARLY_DISCONNECT | .text + 0x19cf38 | .text + 0x19d45c |
| case 0xF  | NO_TRANSFER                       | .text + 0x19cf3c | .text + 0x19d4fc |
| case 0x10 | DISCONNECT_ONLINE                 | .text + 0x19cf40 | .text + 0x19d068 |
| case 0x11 | SAVE_SETTINGS                     | .text + 0x19cf44 | .text + 0x19d0b0 |
| case 0x12 | _unnamed_                         | .text + 0x19cf48 | .text + 0x19d4bc |

The jump table of get_next_state is located at .text + 0x242bcc and has these jumps:

| Case #    | State Name                        | Offset           | Jump offset      |
| --------- | --------------------------------- | ---------------- | ---------------- |
| case 0x0  | GAME_LOAD                         | .text + 0x242bcc | .text + 0x242c18 |
| case 0x1  | SELECT_LANGUAGE                   | .text + 0x242bd0 | .text + 0x242c2c |
| case 0x2  | HOME_MENU                         | .text + 0x242bd4 | .text + 0x242c50 |
| case 0x3  | CONNECT_ONLINE                    | .text + 0x242bd8 | .text + 0x242c90 |
| case 0x4  | _unnamed_                         | .text + 0x242bdc | .text + 0x242cb4 |
| case 0x5  | _unnamed_                         | .text + 0x242be0 | .text + 0x242cc4 |
| case 0x6  | TRANSFER_POKEMON                  | .text + 0x242be4 | .text + 0x242cdc |
| case 0x7  | _unnamed_                         | .text + 0x242be8 | .text + 0x242cf4 |
| case 0x8  | SHOW_GAMES                        | .text + 0x242bec | .text + 0x242d0c |
| case 0x9  | GET_POKEMON                       | .text + 0x242bf0 | .text + 0x242d28 |
| case 0xA  | _unnamed_                         | .text + 0x242bf4 | .text + 0x242d40 |
| case 0xB  | CHECK_IF_USER_CAN_TRANSFER        | .text + 0x242bf8 | .text + 0x242d68 |
| case 0xC  | ASK_TO_TRANSFER                   | .text + 0x242bfc | .text + 0x242d90 |
| case 0xD  | _unnamed_                         | .text + 0x242c00 | .text + 0x242dac |
| case 0xE  | CLEANUP_PREVIOUS_EARLY_DISCONNECT | .text + 0x242c04 | .text + 0x242dd0 |
| case 0xF  | NO_TRANSFER                       | .text + 0x242c08 | .text + 0x242df0 |
| case 0x10 | DISCONNECT_ONLINE                 | .text + 0x242c0c | .text + 0x242e00 |
| case 0x11 | SAVE_SETTINGS                     | .text + 0x242c10 | .text + 0x242e00 |
| case 0x12 | _unnamed_                         | .text + 0x242c14 | .text + 0x242df0 |

An easy way to patch `get_next_state` is with this assembly:

```assembly
mov r0, #<whatever state should be next>
pop {r4, pc}
```

For example to set the next state after `TRANSFER_POKEMON` to always be `NO_TRANSFER`, you would modify .text + 0x242cdc to have:

```assembly
mov r0, #0xf
pop {r4, pc}
```
