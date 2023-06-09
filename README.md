---
typora-root-url: ./ 
---



# TMine键盘扫雷游戏
TMine terminal Minesweeper game,support mouse operation

> Author：vvvvvx
>
> Time：2023-04-07
>
> Address：China



该程序是一个键盘扫雷游戏，只需要键盘敲入行、列即可操作单元格，支持鼠标操作。详解如下。

This is a keyboard Minesweeper game, input the row and column number of cell to operate the cell.Also support mouse operation.

![difficulty](/difficulty.png)

![playing](/playing.png)

![success](/success.png)![failed](/failed.png)

## 命令格式/Command format

命令格式：行+列+命令。

Command format:row+col+CMD

命令包括：

CMD:

1. D-Dig  翻开  [ 可 简化操作 ]/[May simple operate]

    当判断某单元格非雷时，翻开此格

    When you decide the cell is not mine,dit it.

2. F-Flag 标雷

    如某格为雷，此命令标记一个小红旗

    if the cell is mine,mark it as a flag.

    当此格已经标记为雷，输入同样命令，将取消标记

    when the cell is already marked as a mine,this cmd will un-mark it. 

3. T-Test 测试  [ 可 简化操作 ]/[May simple operate]

    假如某单元格周围有2个雷，若你已在其周围标记了2个雷，Test此格，将翻开其周围所有未翻开、未标记为雷的格子。

    If there are two mines arround the cell and you have marked two,Test command will open all un-marking cells arround.

    **Test命令仅适用于已经翻开的单元格。**

    **Test command is only for cells opened.**

4. P-Pending 疑问

    当你不确定此单元格是雷时，做一个问号标记

    When you can't decide if the cell is mine,mark a Pending flag.

命令例子：

Command example:

- CED+Enter：表示翻开C行E列单元格  [ 可 简化操作 ]/[May simple operate]

    Means Dig the cell of C-row and E-column

- CEF+Enter：表示标记C行E列单元格为雷 

    Means Mark the cell of C-row and E-column the mine.

- CET+Enter：表示测试C行E列单元格  [ 可 简化操作 ]/[May simple operate]

    Means Test the cell of C-row and E-column the mine.

- CEP+Enter：表示标记C行E列为问号

    Means Mark Pending the cell of C-row and E-column the mine.

## 简化操作/Simple Operate

为提高速度，当Dig和Test操作时，可简化输入行列，回车即可，无需输入命令。

To impove the speed, Dig and Test operation can only input row and col.

简化命令例子：

Simple Operate example:

1. 当单元格未翻开时/When cell is not opened

    CF+回车：表示翻开C行F列
    
    CF+Enter: Means Dig the cell.Equal to CFD+Enter

2. 当单元格已经翻开时/When cell is opened

    CF+回车：表示测试C行F列

    CF+Enter:Means Test the cell. Equal to CFT+Enter
    
## 鼠标操作 / Mouse Operate
1. 左击 / Left click
    - 当格子未翻开时，翻开此格
    - When cell is not opened,Open the cell.
    - 当格子已翻开时，测试此格
    - When cell is opened,Test the cell.
2. 右击 / Right click
   
    标记此格为雷

    Mark the cell mine
3. 左击红色命令 / Left click Red-CMD
   鼠标左击界面右下红色命令，可直接执行。
![red-cmd](/red-cmd.png)
# tmine

>>>>>>> origin/main
