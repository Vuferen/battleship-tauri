import imp
import cv2
import numpy as np
import array

Board_Position = 0,0,0,0

X_Offset = -15
Y_Offset = -9

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

def positions():
    return pos
        
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
        
    

pos = array.array('l', 100 * [0])

cap = cv2.VideoCapture(1)



def refresh():
    Board_Position = 0,0,0,0

    X_Offset = -15
    Y_Offset = -9

    pos = array.array('l', 100 * [0])

    cap = cv2.VideoCapture(1)

    ret, img = cap.read()
    imgOutput = img
    
    gray = cv2.cvtColor(imgOutput, cv2.COLOR_BGR2GRAY)
    _, binary = cv2.threshold(gray, 150, 255, cv2.THRESH_BINARY_INV)
    blur = cv2.GaussianBlur(gray, (5, 5), 0)
    canny = cv2.Canny(blur, 1, 100)
    ret, mask = cv2.threshold(canny, 100, 255, cv2.THRESH_BINARY)

    contours, hierarchy = cv2.findContours(mask, cv2.RETR_TREE, cv2.CHAIN_APPROX_SIMPLE)

    for index, cnt in enumerate(contours, start = 0):
        area = cv2.contourArea(cnt)
        if(area > 400 and area < 10000):
            img = cv2.drawContours(img, [cnt], -1, (0,255,255), 3)
            x, y, w, h = cv2.boundingRect(cnt)
            x = x - Board_Position[0] + X_Offset
            y = y - Board_Position[1] + Y_Offset
            boatpos = int(x/26), int(y/25), int(round(w/24)), int(round(h/24))
            setPos(pos, boatpos)
 

    for index, cnt in enumerate(contours, start = 0):
        x1,y1 = cnt[0][0]
        approx = cv2.approxPolyDP(cnt, 0.01*cv2.arcLength(cnt, True), True)
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
                Board_Position = x,y,w,h 
                break

    width, height = 200, 200

    pointOne = Board_Position[0], Board_Position[1]
    pointTwo = Board_Position[0] + Board_Position[2], Board_Position[1]
    pointThree = Board_Position[0], Board_Position[1] + Board_Position[3]
    pointFour = Board_Position[0] + Board_Position[2], Board_Position[1] + Board_Position[3]
    pts1 = np.float32(reorder([pointOne, pointTwo, pointThree, pointFour]))
    pts2 = np.float32([[-10, 0], [width + 35, 0], [0 + 0, height], [width, height]])

    matrix = cv2.getPerspectiveTransform(pts1, pts2)
    imgOutput = cv2.warpPerspective(img, matrix, (width, height))

    cv2.imshow("Test", imgOutput)

    print(pos[0], "Test", pos[1])
    
    # if cv2.waitKey(1) == 13:
    
    cv2.destroyAllWindows()
    print(pos)
    return pos
    
cap.release()
cv2.destroyAllWindows()

cv2.waitKey(0)


