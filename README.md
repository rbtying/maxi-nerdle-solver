# maxi-nerdle-solver

A silly little solver for the nerdle/maxi-nerdle style wordle games

Contains the following:

1. `gen_` binaries which generate the full set of possible equations for classic, maxi, and micro nerdles according to their rules. Based heavily on sources from around the internet!

2. a `filter` binary which interactively solves the puzzle with you, suggesting good-but-not-necessarily-perfect guesses and narrowing the search space until the solution is found.

```
cargo run --release --bin filter maxi_nerdle.txt


Reading options from maxi_nerdle.txt
Loaded 2177736 options
2177736 options remaining

Best next guess based on 1,000 randomly-selected equations: 42+9-35=16, information: -255.64874049174452
- 1112/139=8
- 1112/2=556
- 1112/278=4
- 1112/4=278
- 1112/556=2
- 1112/8=139
- 1113/159=7
- 1113/21=53
- 1113/3=371
- 1113/371=3
- 1113/53=21
- 1113/7=159
- 1114/2=557
- 1114/557=2
- 1115/223=5
- 1115/5=223
- 1116/12=93
- 1116/124=9
- 1116/18=62
- 1116/186=6
- 1116/2=558
- 1116/279=4
- 1116/3=372
- 1116/31=36
- 1116/36=31
- ...

Enter your guess (you can use s for ² and c for ³)
42+9-35=16
Enter your mask (G or 2 for green; P or 1 for purple; B or 0 for black)
0100111110
3725 options remaining

Best next guess based on 1,000 randomly-selected equations: 35-18-15=2, information: -2061.982350058851
- 111/3-32=5
- 111/3-35=2
- 112-35*3=7
- 112-3*35=7
- 112/8-3²=5
- 113-21*5=8
- 113-22*5=3
- 113-2³=105
- 113-55*2=3
- 113-5*21=8
- 113-5*22=3
- 115-12=103
- 115-5*23=0
- 115*3-7³=2
- 115/5-23=0
- 115/5-20=3
- 117-57*2=3
- 117-5*23=2
- 118-5*23=3
- 11-1*2*3=5
- 11-2*1*3=5
- 11-2*(3)=5
- 11-2/1*3=5
- 11-3*1*2=5
- 11-3*2-5=0
- ...

Enter your guess (you can use s for ² and c for ³)
35-18-15=2
Enter your mask (G or 2 for green; P or 1 for purple; B or 0 for black)
2121001021
3 options remaining

Best next guess: 31-5*2*3=1, information: 0
- 31-2*5*3=1
- 31-3*5*2=1
- 31-5*2*3=1

Enter your guess (you can use s for ² and c for ³)
31-5*2*3=1
Enter your mask (G or 2 for green; P or 1 for purple; B or 0 for black)
2222222222
1 options remaining

Best next guess: 31-5*2*3=1, information: 0
- 31-5*2*3=1
```
