var loopStart = clock();

for (var i = 0; i < 10000000; i = i+1) {
    if (i > 10) break;
    for (var j = 0; j < 10000000; j = j+1) {
        print("" + j + ", " + i);
        if (j > 3) break;
    }
}
var loopTime = clock() - loopStart;

print("loop:", loopTime);