export const useCustomToast = () => {
  const toast = useToast();
  const throwError = (text: string) => {
    toast.add({
      title: "Something went wrong!",
      description: text,
      color: "error",
    })
  }

  const throwSuccess = (text: string) => {
    toast.add({
      title: "Success!",
      description: text,
      color: "success",
    })
  }

  const throwWarning = (text: string) => {
    toast.add({
      title: "Attention!",
      description: text,
      color: "warning",
    })
  }
  return { throwSuccess, throwWarning, throwError };
}
