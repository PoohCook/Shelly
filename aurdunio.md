The code in Seaside.ino is arduino from a previous project. In that project there were 235 LEDS broken up into 36 different sections. The leds were mapped via sectionMap that determines which LE$D is in which section.

The new code is the rust project we are in. It has a LightPorts controller that governs access to teh LEDS and breaks them up into "blades" (same as sections above) and there is no need for mapping as blades are sequentially addressed.

I want to migrate the code form Seaside.ino into the rust project. Instead of running Timer interrupts to drive the effects, I want to use the loop in main.rs which is currently doing very little otehr than running the LightPorts engine. Please use LightPorts.setBalde() to set the collor of any blade by index. Also, always set the blink parameter to false.


ShellFire Effect is about emulating the visual effect of a fire. Each blade of the shell is arrange in a Spiral with the higher numbered blades reaching up highest. The effect should emulate heat pops spiralling out in the fire. The pops should occur anywhere and spin out towards higher blade nubers but diminish as they spiral out. The inner core (lower blade numbers) should always bne hotest


