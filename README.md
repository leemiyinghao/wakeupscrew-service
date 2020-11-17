# WakeUpScrew

WakeUpScrew is a Line targeted Chatbot project, based on NLP technology.  
It's aimed to build a fun and accurate chat experience in daily IM chat.

## Refactor

The repo here is a Rust refactor version of original [WakeUpScrew](https://github.com/leemiyinghao/wakeupscrew), which was written in Python, flask, and Tensorflow.

The reason to refactor from Python to Rust is due to the termination of self-hosting server and limitation of the new hosting environment. After refactoring, it's memory usage shrink from around 20GB to 500MB. Concurrency performance also gain a lot.

In process of refactoring, the NLP reply core became an [independent craft](https://github.com/leemiyinghao/vec2seq-rust) to achieve decoupling.

## Chat Demonstration

![](https://github.com/leemiyinghao/wakeupscrew-service/blob/main/20gJ8PNDs4Ea9JkYiuBcw7.jpg)
![](https://github.com/leemiyinghao/wakeupscrew-service/blob/main/2XRlJnlT7k8Jo7sM0xWSJI.jpg)
![](https://github.com/leemiyinghao/wakeupscrew-service/blob/main/4DMwI4wlJz4FFECo4gwBuh.jpg)
![](https://github.com/leemiyinghao/wakeupscrew-service/blob/main/6zX84v3OSqUCX2B05enNOt.jpg)

# Ref
[PTT Gossiping Corpus fit for training](https://github.com/zake7749/Gossiping-Chinese-Corpus)
[WakeUpScrew(Python Version)](https://github.com/leemiyinghao/wakeupscrew)
[Vec2Seq](https://github.com/leemiyinghao/vec2seq-rust)
