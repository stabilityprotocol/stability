type Ok = {
  success: true;
  data: string;
};

type Err = {
  success: false;
  error: string;
};

export const Ok = (data: string): Ok => ({
  success: true,
  data,
});

export const Err = (error: string): Err => ({
  success: false,
  error,
});

export type Result = Ok | Err;
