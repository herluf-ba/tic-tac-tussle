# tic-tac-tussle
A small online tic tac toe game written in rust for a tutorial series

### TODO Server
- [ ] Make lobby id generation (short hash)
- [ ] Make a way for client to create a lobby
- [ ] Make a way for client to join a lobby
- [ ] Client auth token generation. Token contains user id and lobby
- [ ] Make lobby game event delegation

### TODO Client
 Game Loop
*Join game*: Enter game code + Enter name -> Game begins
*Create game*: Enter name -> Send game code to friend -> Game Begins

- [ ] Figure out how to do an input field
- [ ] Style input field
- [ ] Use input field to make "Name" and "Gamecode" fields
- [ ] Make stages in the bevy game (Initial, CreateGame, JoinGame, Playing)
- [ ] Use run criteria for systems (or conditionally add and remove them)

*Visuals*
- [ ] Scale world (1px -> 1 in game unit)
- [ ] Add background
- [ ] Add display of tics and tacs
- [ ] Add mouse hover effect on empty tiles
- [ ] Add click event
- [ ] Add display of player names
- [ ] Add highlighing of current active player
- [ ] Add celebration of winner
- [ ] Add replay in lobby option

