namespace AeneasMicro

def clamp (byte : UInt8) : UInt8 :=
  byte &&& 248

def mask_low (byte : UInt8) : UInt8 :=
  byte &&& 127

def process (input : UInt8) : UInt8 :=
  mask_low (clamp input)

end AeneasMicro
