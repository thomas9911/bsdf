import bsdf

# my_object = {
#     "hallo": 1
# }

# my_object = 3.1415
# my_object = "hallo"
# my_object = """
# Lorem ipsum dolor sit amet, consectetur adipiscing elit. Duis id ante velit. Aenean euismod, ipsum a varius finibus, eros erat tincidunt ligula, non malesuada ex ipsum et tellus. Cras id convallis mauris, mattis porttitor nulla. In urna orci, faucibus ut consequat eleifend, vulputate ac elit. Integer gravida porta arcu, id volutpat libero lobortis at. Aenean bibendum eleifend auctor. Sed lectus purus, aliquet non purus ut, feugiat tristique leo. Praesent ut leo blandit, vulputate ex sit amet, venenatis libero. Curabitur vehicula ut enim sed posuere. Aliquam nec elit fringilla, aliquet lectus sed, suscipit quam. Vivamus malesuada ligula eu luctus finibus. Proin euismod sem sit amet eros euismod rhoncus.
# """
# my_object = "".join(["x" for x in range(250)])
# my_object = {
#     "test": 1,
#     "nested": {
#         "nested": True,
#         "list": [-1, False, 123456789],
#         "data": "some text"
#     }
# }

my_object = bytes([1,2,3,4,5,6,7,8,9,0])
# print(len(my_object))
bb = bsdf.encode(my_object, compression=2, use_checksum=True)

print(bb)

# print([x for x in b"BSDF\x02\x02s\xfd\x1d&\x00\x00\x00\x00\x00\x00"])

print([x for x in bb])
