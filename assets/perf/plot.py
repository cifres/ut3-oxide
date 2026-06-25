import matplotlib.pyplot as plt
from numpy import average

t5 = [998, 132, 138, 149, 124, 127, 122, 148, 143, 141, 129, 146, 159, 204,
      154, 184, 146, 169, 148, 163, 168, 176, 144, 144, 127, 128, 121, 159,
      112, 153, 134, 102, 110, 109, 115, 109, 119, 101, 98, 116, 101, 208, 109,
      228, 121, 195, 113, 202, 263, 282, 142, 128, 98, 77, 43]

t7 = [9129, 1743, 1522, 1098, 1778, 1491, 1323, 729, 1472, 1636, 2012, 2318,
      1122, 1114, 1616, 824, 1326, 1197, 906, 986, 1796, 1530, 1710, 1610,
      1738, 1199, 1357, 1248, 919, 756, 884, 617, 764, 1004, 683, 650, 677,
      600, 1650, 469, 650, 474, 981, 351, 475, 135, 181, 44, 66, 43, 43, 39,
      45]

mini = [min(t5[1:]), min(t7[1:])]
maxi = [max(t5[1:]), max(t7[1:])]
avg = [average(t5[1:]), average(t7[1:])]
print(f"{mini=} {maxi=} {avg[0]=:.5} {avg[1]=:.5}")

plt.figure(1)
plt.suptitle("Depth 5 (top) versus Depth 7 (bottom)")
plt.subplot(211)
plt.ylabel("Duration (μs)")
plt.plot(t5[1:])

plt.subplot(212)
plt.ylabel("Duration (μs)")
plt.xlabel("Turn count (arbitrary units)")
plt.plot(t7[1:])

plt.show()

# plt.plot(t7[1:])
# plt.ylabel("Duration (μs)")
# plt.xlabel("Turn count")
# plt.show()
#
# plt.plot(t7[1:])
# plt.show()
