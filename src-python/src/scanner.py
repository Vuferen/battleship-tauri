import cv2
import numpy as np
import array
import sys

def reorder(pts):
    pts = np.array(pts).reshape((4, 2))
    pts_new = np.zeros((4, 1, 2), np.int32)
    add = pts.sum(1)
    pts_new[0] = pts[np.argmin(add)]
    pts_new[3] = pts[np.argmax(add)]
    diff = np.diff(pts, axis=1)
    pts_new[1] = pts[np.argmin(diff)]
    pts_new[2] = pts[np.argmax(diff)]
    return pts_new

def clamp(num, min_value, max_value):
   return max(min(num, max_value), min_value)

def resetPos(pos):
    i = 0
    while i < len(pos):
        i = i + 1
        pos[i - 1] = 0
        
def setPos(pos, boatPos):
    w = int(boatPos[2])
    h = int(boatPos[3])
    
    index = 0
    secondIndex = 0
    
    index = clamp(boatPos[0] + boatPos[1] * 10, 0, 99)


    pos[index] = 1

    i = 0
    e = 0
    if(w > 1):
        while i < w:
            i += 1
            secondIndex = clamp(index + i - 1, 0, 99)
            pos[secondIndex] = 1
    if(h > 1):
        while e < h:
            e += 1
            secondIndex = clamp(index + (e - 1) * 10, 0, 99)
            pos[secondIndex] = 1
    # print(str(index) + " and " + str(secondIndex))
        
def getShapePoints(approx, img2, positions):
    n = approx.ravel()
    i = 0
    c = 0

    if len(n) != 8:
        return
  
    for j in n :
        if(i % 2 == 0):
            x = n[i]
            y = n[i + 1]               
            positions[c] = [x, y]
            c += 1
            if c >= 4:
                
                # positions[3][0] += 100
                break

        i = i + 1
    return
    

def refresh(Board_Position, X_Offset, Y_Offset, pos, cap, corners, boat_offset):

    ret, img = cap.read()
    imgOutput = img
    
    gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
    _, binary = cv2.threshold(gray, 170, 255, cv2.THRESH_BINARY_INV)
    blur = cv2.medianBlur(gray, 15)
    sharpen_kernel = np.array([[-1, -1, -1], [-1, 9, -1], [-1,-1,-1]])
    sharpen = cv2.filter2D(blur, -1, sharpen_kernel)
    thresh = cv2.threshold(sharpen, 130, 200, cv2.THRESH_BINARY_INV)[1]
    kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (3,3))
    close = cv2.morphologyEx(thresh, cv2.MORPH_CLOSE, kernel, iterations=2)
    
    canny = cv2.Canny(close, 1, 100)
    #ret, mask = cv2.threshold(canny, 100, 255, cv2.THRESH_BINARY)

    # cv2.imshow("Cal", close)
    contours, hierarchy = cv2.findContours(close, cv2.RETR_TREE, cv2.CHAIN_APPROX_SIMPLE)

    approx = None
    for index, cnt in enumerate(contours, start = 0):
        x1,y1 = cnt[0][0]
        approx = cv2.approxPolyDP(cnt, 0.1 *cv2.arcLength(cnt, True), True)
        area = cv2.contourArea(cnt)
        if(area > 50000):
            if len(approx) == 4:
                x, y, w, h = cv2.boundingRect(cnt)
                ratio = float(w)/h
                if ratio >= 0.9 and ratio <= 1.1:
                    img = cv2.drawContours(img, [cnt], -1, (0,255,255), 3)
                    cv2.putText(img, 'Square', (x1, y1), cv2.FONT_HERSHEY_SIMPLEX, 0.6, (255, 255, 0), 2)
                else:
                    cv2.putText(img, 'Rectangle', (x1, y1), cv2.FONT_HERSHEY_SIMPLEX, 0.6, (0, 255, 0), 2)
                    img = cv2.drawContours(img, [cnt], -1, (0,255,0), 3)
                
                Board_Position[0] = x
                Board_Position[1] = y
                Board_Position[2] = w
                Board_Position[3] = h
                
                break

    width, height = 500, 500
    boat_x = boat_offset[0]
    boat_y = boat_offset[1]
    getShapePoints(approx, imgOutput, corners)
    # print(str(corners))
    temp = corners[0]
    corners[0] = corners[1]
    corners[1] = temp
    pts1 = np.float32(reorder(corners))
    # print(corners)
    # [[137, 92], [509, 91], [44, 479], [602, 479]]
    boardBorderOffset = 0
    # print(boardBorderOffset)
    #print("B: " + str(pointThree[0]))
    # print("1: " + pointOne.0 + pointOne.1, "2: " + pointTwo.0 + pointTwo.1, "3: " + pointThree.0 + pointThree.1, "4 " + pointFour.0 + pointFour.1)
    pts2 = np.float32([[boat_x, boat_y], [width - boat_x, boat_y], [boat_x, height - boat_y - boardBorderOffset], [width - boat_x, height - boat_y - boardBorderOffset]])

    matrix = cv2.getPerspectiveTransform(pts1, pts2)
    imgOutput = cv2.warpPerspective(img, matrix, (width, height))

    imgWarped = imgOutput
    grayWarped = cv2.cvtColor(imgWarped, cv2.COLOR_BGR2GRAY)
    _, binaryWarped = cv2.threshold(grayWarped, 150, 255, cv2.THRESH_BINARY_INV)
    blurWarped = cv2.medianBlur(grayWarped, 15)
    sharpen_kernelWarped = np.array([[-1, -1, -1], [-1, 9, -1], [-1,-1,-1]])
    sharpenWarped = cv2.filter2D(blurWarped, -1, sharpen_kernelWarped)
    threshWarped = cv2.threshold(sharpenWarped, 130, 200, cv2.THRESH_BINARY_INV)[1]
    kernelWarped = cv2.getStructuringElement(cv2.MORPH_RECT, (3,3))
    closeWarped = cv2.morphologyEx(threshWarped, cv2.MORPH_CLOSE, kernel, iterations=2)
    cannyWarped = cv2.Canny(closeWarped, 1, 100)
    #ret, mask = cv2.threshold(canny, 100, 255, cv2.THRESH_BINARY)

    # contoursWarped, hierarchyWarped = cv2.findContours(closeWarped, cv2.RETR_TREE, cv2.CHAIN_APPROX_SIMPLE)

    # for index, cnt in enumerate(contoursWarped, start = 0):
    #     area = cv2.contourArea(cnt)
    #     if(area > 2000 and area < 20000):
    #         imgWarped = cv2.drawContours(imgWarped, [cnt], -1, (0,255,255), 3)
    #         x, y, w, h = cv2.boundingRect(cnt)
    #         boatpos = int(x/44), int(y/44), int(round(w/44)), int(round(h/44))

    #         setPos(pos, boatpos)

    for i in range(0,10):
        for j in range(0,10):
            if closeWarped[i*42+64, j*41+62] > 0:
                cv2.circle(closeWarped, (j*41+62, i*42+64), 5, (0,0,0))
                
            else:
                cv2.circle(closeWarped, (j*41+62, i*42+64), 5, (255,255,255))
                pos[i*10+j] = 1

    


    # cv2.imshow("Test", imgOutput)
    # cv2.imshow("Warped", closeWarped)


    
    # if cv2.waitKey(1) == 13:
    
    # cv2.destroyAllWindows()
    # for i in range(0,10):
    #     for j in range (0,10):
    #         print(pos[j+i*10], sep = ", ", end = " ")
    #     print("")
    
    return pos


# Board_PositionGlobal = ([0, 0, 0, 0])

# X_OffsetGlobal = -15
# Y_OffsetGlobal = -9

# capGlobal = cv2.VideoCapture(1)

# posGlobal = array.array('l', 100 * [0])

# BoatOffset = [-15, -15]

# corners = [[0,0], [0,0], [0,0], [0,0]]

def GetShips(deviceId):
    Board_PositionGlobal = ([0, 0, 0, 0])

    X_OffsetGlobal = 0
    Y_OffsetGlobal = 0

    capGlobal = cv2.VideoCapture(deviceId)

    posGlobal = array.array('l', 100 * [0])

    BoatOffset = [0, 0]

    corners = [[0,0], [0,0], [0,0], [0,0]]
    # while True:
        # resetPos(posGlobal)
    print(refresh(Board_PositionGlobal, X_OffsetGlobal, Y_OffsetGlobal, posGlobal, capGlobal, corners, BoatOffset))

    # while True:
    #     resetPos(posGlobal)
    #     refresh(Board_PositionGlobal, X_OffsetGlobal, Y_OffsetGlobal, posGlobal, capGlobal, corners, BoatOffset)
    #     if cv2.waitKey(1) == 13:
    #         break

if __name__ == "__main__":
    GetShips(int(sys.argv[1]))
# capGlobal.release()
# cv2.destroyAllWindows()

# cv2.waitKey(0)


